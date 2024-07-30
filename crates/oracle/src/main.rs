use std::{fs, str::FromStr, sync::Arc};

use cosmrs::{
    auth::BaseAccount,
    crypto::secp256k1::SigningKey,
    proto::{
        cosmos::auth::v1beta1::{BaseAccount as ProtoBaseAccount, QueryAccountRequest},
        prost::Message,
    },
    AccountId,
    Denom,
};
use eyre::{eyre, OptionExt, Result};
use filament_hub_core::{
    playbook::{
        Auth,
        Budget,
        ConversionDescription,
        ConversionMechanism,
        ConversionProofMechanism,
        PayoutMechanism,
        SegmentDescription,
        SegmentKind,
        SegmentProofMechanism,
    },
    CallMessage,
    Playbook,
};
use futures::FutureExt as _;
use jsonrpsee::{core::client::ClientT, rpc_params};
use neutron::state::Campaign;
use sov_cli::wallet_state::PrivateKeyAndAddress;
use sov_ledger_apis::rpc::client::RpcClient;
use sov_modules_api::{utils::generate_address, CryptoSpec, PrivateKey, PublicKey, Spec};
use sov_rollup_interface::rpc::QueryMode;
use tendermint_rpc::{Client, WebSocketClient};
use tokio::sync::{watch, Mutex};
use tonic::transport::Endpoint;
use tracing::debug;

mod config;
mod hub;
mod outpost;

// XXX: to initialize pull all campaigns from outpost that are Created, Funded,
//      Indexing, Attesting?

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = argh::from_env::<config::Args>();

    ///// Outpost setup

    let channel = Endpoint::try_from(args.outpost_grpc_endpoint)?
        .connect()
        .await?;

    let (outpost_client, outpost_driver) =
        WebSocketClient::new("ws://localhost:26657/websocket").await?;
    let mut outpost_driver_handle = tokio::spawn(async move { outpost_driver.run().await }).fuse();

    let signing_key_bytes = {
        let mut raw = fs::read(args.outpost_secret_key_file.clone())?;
        raw.truncate(raw.len() - 1); // why???
        hex::decode(raw)?
    };
    let outpost_account_id = AccountId::from_str(
        SigningKey::from_slice(&signing_key_bytes)
            .unwrap()
            .public_key()
            .account_id("neutron")
            .unwrap()
            .as_ref(),
    )?;
    debug!("outpost account_id: {:}", outpost_account_id);

    let outpost_account: BaseAccount = ProtoBaseAccount::decode(
        cosmrs::proto::cosmos::auth::v1beta1::query_client::QueryClient::new(channel.clone())
            .account(QueryAccountRequest {
                address: outpost_account_id.to_string(),
            })
            .await?
            .into_inner()
            .account
            .ok_or(eyre!("no account found"))?
            .value
            .as_slice(),
    )?
    .try_into()?;
    let outpost_account = Arc::new(Mutex::new(outpost_account));
    debug!("outpost account: {:?}", outpost_account);

    let op = Arc::new(Mutex::new(outpost::Outpost::new(
        outpost_account.clone(),
        outpost_account_id.clone(),
        signing_key_bytes,
        args.outpost_chain_id,
        Denom::from_str("untrn")?,
        args.outpost_addr.clone(),
        outpost_client.clone(),
        channel.clone(),
    )));

    ///// Hub setup

    // let hclient = SovClient::new(&args.hub_rpc_endpoint);
    let hws = hub::rpc_client("ws://localhost:12345".to_string()).await;

    debug!("hclient: {:?}", hws.get_head(QueryMode::Compact).await);

    let key_and_address: PrivateKeyAndAddress<hub::FilaSpec> = serde_json::from_str(
        &std::fs::read_to_string(args.hub_secret_key_file.clone())
            .expect("Unable to read file to string"),
    )?;
    debug!("hub address: {:?}", key_and_address.address);
    let hub_pkh = key_and_address
        .private_key
        .pub_key()
        .secure_hash::<<<hub::FilaSpec as Spec>::CryptoSpec as CryptoSpec>::Hasher>()
        .0;
    let hub_account = match hws
        .request("accounts_getAccount", rpc_params!(hub_pkh.to_vec()))
        .await?
    {
        hub::AccResp::AccountExists { addr: _, nonce } => Arc::new(Mutex::new(hub::Account {
            pkh: hub_pkh,
            nonce,
        })),
        _ => Arc::new(Mutex::new(hub::Account {
            pkh: hub_pkh,
            nonce: 0,
        })),
    };

    let hb = Arc::new(Mutex::new(hub::Hub::new(
        "ws://localhost:12345".to_string(),
        hub_account.clone(),
        key_and_address.private_key.clone(),
    )));

    let (last_block_sender, last_block_recv) = match outpost_client.latest_block().await {
        Ok(res) => {
            // println!("block: {:?}", res.block_id);
            Ok(watch::channel(res.block))
        },
        Err(e) => Err(eyre!("failed to poll latest block: {}", e)),
    }?;

    let (last_slot_sender, last_slot_recv) = match hws.get_head(QueryMode::Compact).await {
        Ok(Some(res)) => {
            println!("slot: {:?}", res.number);
            Ok(watch::channel(res.number))
        },
        Ok(None) => Err(eyre!("got no slot back")),
        Err(e) => Err(eyre!("failed to poll latest slot: {}", e)),
    }?;

    let (last_campaign_sender, last_campaign_recv) = watch::channel(0u64);

    let mut block_handle = tokio::spawn(outpost::blocks(last_block_recv.clone())).fuse();
    let mut latest_block_handle = tokio::spawn(outpost::latest_block(
        outpost_client.clone(),
        last_block_sender.clone(),
    ))
    .fuse();
    let mut latest_slot_handle = tokio::spawn(hub::latest_slot(
        "ws://localhost:12345".to_string(),
        last_slot_sender.clone(),
    ))
    .fuse();
    let mut outpost_account_handle = tokio::spawn(outpost::base_account_watcher(
        channel.clone(),
        outpost_account_id.clone(),
        outpost_account.clone(),
        last_block_recv.clone(),
    ))
    .fuse();

    let mut hub_account_handle = tokio::spawn(hub::account_watcher(
        hws.clone(),
        hub_pkh,
        hub_account.clone(),
        last_slot_recv.clone(),
    ))
    .fuse();

    let mut outpost_contract_handle = tokio::spawn(outpost::contract_watcher(
        outpost_client.clone(),
        args.outpost_addr,
        last_campaign_sender,
    ))
    .fuse();
    let mut outpost_campaigns_handle = tokio::spawn(outpost_campaigns(
        op.clone(),
        hb.clone(),
        last_campaign_recv,
    ))
    .fuse();

    loop {
        tokio::select! {
            join = &mut outpost_driver_handle => join??,
            join = &mut block_handle => join??,
            join = &mut latest_block_handle => join??,
            join = &mut latest_slot_handle => join??,
            join = &mut outpost_account_handle => join??,
            join = &mut outpost_contract_handle => join??,
            join = &mut outpost_campaigns_handle => join??,
            join = &mut hub_account_handle => join??,
            else => break
        }
    }

    Ok(())
}

//
pub async fn outpost_campaigns(
    op: Arc<Mutex<outpost::Outpost>>,
    hb: Arc<Mutex<hub::Hub>>,
    mut lcr: watch::Receiver<u64>,
) -> Result<()> {
    debug!("outpost_campaign");
    loop {
        tokio::select! {
            Ok(()) = lcr.changed() => {
                let campaign_id: u64;
                {
                    campaign_id = *lcr.borrow();
                }
                debug!("campaign_id {:} touched", campaign_id);

                let oc: Campaign;
                {
                    let mut o = op.lock().await;
                    let outpst_campaign = o.pull_campaign(campaign_id).await?;
                    debug!("outpost campaign: {:?}", outpst_campaign);
                    oc = outpst_campaign;
                }

                {
                    let h = hb.lock().await;
                    let cc = outpost_to_hub_campaign(oc)?;
                    debug!("sending hub campaign");
                    h.create_campaign(cc).await?;
                }
            },
            else => break,
        }
    }
    Ok(())
}

pub fn outpost_to_hub_campaign(c: Campaign) -> Result<CallMessage<hub::FilaSpec>> {
    let budget = outpost_to_hub_budget(c.budget.ok_or_eyre("no budget found")?)?;
    let hpm = match c.payout_mech {
        neutron::state::PayoutMechanism::ProportionalPerConversion => {
            PayoutMechanism::ProportionalPerConversion
        },
    };
    let hsm = match c.segment_desc.kind {
        neutron::state::SegmentKind::GithubAllContributors => SegmentKind::GithubAllContributors,
        neutron::state::SegmentKind::GithubTopNContributors(n) => {
            SegmentKind::GithubTopNContributors(n)
        },
    };
    let hspm = match c.segment_desc.proof {
        neutron::state::SegmentProofMechanism::Ed25519Signature => {
            SegmentProofMechanism::Ed25519Signature
        },
    };
    let hcm = match c.conversion_desc.kind {
        neutron::state::ConversionMechanism::Social(neutron::state::Auth::Github) => {
            ConversionMechanism::Social(Auth::Github)
        },
    };
    let hcpm = match c.conversion_desc.proof {
        neutron::state::ConversionProofMechanism::Ed25519Signature => {
            ConversionProofMechanism::Ed25519Signature
        },
    };
    let out = CallMessage::<hub::FilaSpec>::CreateCampaign {
        origin: "neutron-1".to_string(),
        origin_id: c.id,
        indexer: generate_address::<hub::FilaSpec>("indexer"), /* XXX: outpost address does not
                                                                * fit
                                                                * into hub schema */
        attester: generate_address::<hub::FilaSpec>("atter"), /* XXX: outpost address does not
                                                               * fit
                                                               * into hub schema */
        playbook: Playbook {
            budget,
            segment_description: SegmentDescription {
                kind: hsm,
                sources: c.segment_desc.sources,
                proof: hspm,
            },
            conversion_description: ConversionDescription {
                kind: hcm,
                proof: hcpm,
            },
            payout: hpm,
            ends_at: c.ends_at as u128,
        },
    };
    Ok(out)
}

pub fn outpost_to_hub_budget(b: neutron::state::CampaignBudget) -> Result<Budget> {
    let fee = filament_hub_stf::genesis::Coins {
        amount: u64::try_from(b.fee.amount.u128()).unwrap(), // XXX: u64 but comsos coin uses u128
        token_id: sov_bank::TokenId::from_const_slice([0; 32]), // XXX: does not line up with hub
    };

    let incentives = filament_hub_stf::genesis::Coins {
        amount: u64::try_from(b.incentives.amount.u128()).unwrap(), /* XXX: u64 but comsos coin
                                                                     * uses u128 */
        token_id: sov_bank::TokenId::from_const_slice([0; 32]), // XXX: does not line up with hub
    };

    Ok(Budget { fee, incentives })
}
