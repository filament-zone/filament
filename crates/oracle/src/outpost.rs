use std::{str::FromStr, sync::Arc};

use cosmrs::{
    auth::BaseAccount,
    cosmwasm::MsgExecuteContract,
    crypto::secp256k1::SigningKey,
    proto::{
        cosmos::auth::v1beta1::{BaseAccount as ProtoBaseAccount, QueryAccountRequest},
        cosmwasm::{self, wasm::v1::QuerySmartContractStateResponse},
        prost::Message,
        Any,
    },
    tx::{self, Msg, SignDoc, SignerInfo},
    AccountId,
    Coin,
    Denom,
};
use eyre::{eyre, OptionExt, Result};
use futures::StreamExt;
use neutron::{
    msg::{ExecuteMsg, GetCampaignResponse},
    state::{Campaign, CampaignStatus},
};
use serde_json::to_vec;
use tendermint::block;
use tendermint_rpc::{
    event::{Event, EventData},
    query::{EventType, Query},
    Client,
    SubscriptionClient as _,
    WebSocketClient,
};
use tokio::sync::{watch, Mutex};
use tonic::transport::Channel;
use tracing::debug;

pub struct Outpost {
    pub client: WebSocketClient,
    pub channel: Channel,
    pub account: Arc<Mutex<BaseAccount>>,
    pub account_id: AccountId,
    pub signing_key: SigningKey,
    pub chain_id: cosmrs::tendermint::chain::Id,
    pub gas_denom: Denom,
    pub contract: AccountId,
}

impl Outpost {
    pub fn new(
        account: Arc<Mutex<BaseAccount>>,
        account_id: AccountId,
        signing_key: Vec<u8>,
        chain_id: String,
        gas_denom: Denom,
        contract: String,
        client: WebSocketClient,
        channel: Channel,
    ) -> Self {
        Outpost {
            account,
            account_id,
            signing_key: SigningKey::from_slice(&signing_key)
                .expect("--outpost-secret-key-file should point to a valid outpost signing key"),
            chain_id: cosmrs::tendermint::chain::Id::from_str(&chain_id)
                .expect("--outpost-chain-id should set a valid chain id"),
            gas_denom,
            client,
            channel,
            contract: AccountId::from_str(&contract).unwrap(),
        }
    }

    pub async fn pull_campaign(&mut self, campaign_id: u64) -> Result<Campaign> {
        debug!("pull_outpost_campaign({:})", campaign_id);
        let mut client = cosmwasm::wasm::v1::query_client::QueryClient::new(self.channel.clone());

        let value = neutron::msg::QueryMsg::GetCampaign { id: campaign_id };

        let res: QuerySmartContractStateResponse = client
            .smart_contract_state(cosmwasm::wasm::v1::QuerySmartContractStateRequest {
                address: self.contract.to_string(),
                query_data: serde_json::to_vec(&value)?,
            })
            .await?
            .into_inner();
        let cd: GetCampaignResponse = serde_json::from_slice(&res.data)?;

        debug!("campaign {:?}", cd);
        // self.outpost_campaigns.insert(campaign_id, cd.campaign);

        Ok(cd.campaign)
    }

    pub async fn register_segment(&self, campaign_id: u64, size: u64) -> Result<()> {
        let msg = self.register_segment_msg(campaign_id, size);
        let tx = self.mk_tx(msg.to_any()?).await?;

        let res = self.client.broadcast_tx_sync(tx).await?;
        debug!("register_segment broadcast_tx_sync() response {:?}", res);

        Ok(())
    }

    fn register_segment_msg(&self, campaign_id: u64, size: u64) -> MsgExecuteContract {
        MsgExecuteContract {
            sender: self.account_id.clone(),
            contract: self.contract.clone(),
            msg: to_vec(&ExecuteMsg::RegisterSegmentMsg {
                id: campaign_id,
                size,
            })
            .unwrap(),
            funds: vec![],
        }
    }

    async fn mk_tx(&self, msg: Any) -> Result<Vec<u8>> {
        let body = tx::Body::new(vec![msg], "", 0u16);
        let acc: BaseAccount;
        {
            acc = self.account.lock().await.clone();
        }
        let signer = SignerInfo::single_direct(acc.pubkey, acc.sequence);
        let gas_limit = 100_000u64; // XXX
        let gas_coin = Coin {
            amount: 10000u128,
            denom: self.gas_denom.clone(),
        };
        let auth_info = signer.auth_info(tx::Fee::from_amount_and_gas(gas_coin, gas_limit));
        let sign_doc = SignDoc::new(&body, &auth_info, &self.chain_id, acc.account_number)?;
        let signed = sign_doc.sign(&self.signing_key)?;

        signed.to_bytes()
    }
}

// Each new block, sync the outpost account state
pub async fn base_account_watcher(
    channel: Channel,
    account_id: AccountId,
    acc: Arc<Mutex<BaseAccount>>,
    mut lbr: watch::Receiver<block::Block>,
) -> Result<()> {
    debug!("base account");
    loop {
        tokio::select! {
            Ok(()) = lbr.changed() => {
                debug!("pulling latest base account");
                let a: BaseAccount = ProtoBaseAccount::decode(
                    cosmrs::proto::cosmos::auth::v1beta1::query_client::QueryClient::new(channel.clone())
                        .account(QueryAccountRequest {
                            address: account_id.to_string(),
                        })
                        .await?
                        .into_inner()
                        .account
                        .ok_or(eyre!("no account found"))?
                        .value
                        .as_slice(),
                )?.try_into()?;
                let mut c = acc.lock().await;
                *c = a;
            },
            else => break,
        }
    }
    Ok(())
}

// Push out new blocks
pub async fn latest_block(client: WebSocketClient, lbs: watch::Sender<block::Block>) -> Result<()> {
    debug!("latest_block");
    let mut events = client.subscribe(EventType::NewBlock.into()).await?;

    while let Some(Ok(Event {
        data: EventData::LegacyNewBlock { block: Some(b), .. },
        ..
    })) = events.next().await
    {
        lbs.send(*b)?;
    }

    Ok(())
}

// Takes an outpost address and checks each transaction included in a block if it
// interacted with the address and emitted an `campaign_id`.
pub async fn contract_watcher(
    client: WebSocketClient,
    contract: String,
    lcs: watch::Sender<u64>,
) -> Result<()> {
    debug!("starting contract watcher for: {:}", contract);
    let q = Query::from(EventType::Tx).and_eq("execute._contract_address", contract);
    let mut events = client.subscribe(q).await?;

    while let Some(Ok(Event {
        data: EventData::Tx { tx_result },
        events: Some(events),
        ..
    })) = events.next().await
    {
        if !events.contains_key("wasm.campaign_id") {
            debug!("events do no contain wasm.campaign_id");
            continue;
        } else if !events.contains_key("wasm.campaign_status") {
            debug!("events do no contain wasm.campaign_status");
            continue;
        }

        let campaign_id = events["wasm.campaign_id"]
            .first()
            .ok_or_eyre("wasm.campaign_id attribute not found")?;
        // let campaign_status: CampaignStatus = serde_json::from_str(
        // events["wasm.campaign_status"]
        // .first()
        // .ok_or_eyre("wasm.campaign_id attribute not found")?,
        // )?;
        let campaign_status = events["wasm.campaign_status"]
            .first()
            .ok_or_eyre("wasm.campaign_id attribute not found")?;
        debug!("campaign status {:}", campaign_status);

        if *campaign_status == CampaignStatus::Created.to_string() {
            debug!("campaign id {:} created, no need to act yet", campaign_id);
            continue;
        }

        debug!(
            "[{:}] campaign id {:} was touched",
            tx_result.height, campaign_id
        );
        lcs.send(campaign_id.parse::<u64>()?)?;
    }

    Ok(())
}

pub async fn blocks(mut lbr: watch::Receiver<block::Block>) -> Result<()> {
    debug!("blocks");
    loop {
        tokio::select! {
            Ok(()) = lbr.changed() => {
                let height: u64;
                {
                    let latest_block = lbr.borrow().clone();
                    height = latest_block.header().height.into();
                }
                debug!("h: {:}", height);
            },
            else => break,
        }
    }
    Ok(())
}
