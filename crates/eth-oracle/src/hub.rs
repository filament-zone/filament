use std::sync::Arc;

use eyre::{eyre, Context, Result};
use filament_hub_core::{campaign::Campaign, CallMessage, Event, Indexer};
use futures::StreamExt;
use jsonrpsee::{core::client::ClientT, rpc_params};
use reqwest::ClientBuilder;
use sov_ledger_json_client::types::IntOrHash;
use sov_modules_api::{
    default_spec::DefaultSpec,
    execution_mode::Native,
    rest::utils::ResponseObject,
    transaction::{PriorityFeeBips, Transaction, UnsignedTransaction},
    CredentialId,
    CryptoSpec,
    PublicKey,
    Spec,
};
use sov_risc0_adapter::Risc0Verifier;
use tokio::sync::{watch, Mutex};
use tracing::{info, trace};

pub type FilaSpec = DefaultSpec<Risc0Verifier, Risc0Verifier, Native>;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct NonceResponse {
    key: CredentialId,
    value: Option<u64>,
}

pub struct Hub {
    rpc_url: String,
    rpc_client: reqwest::Client,
    json_client: jsonrpsee::http_client::HttpClient,
    ledger_client: sov_ledger_json_client::Client,
    sequencer_client: sov_sequencer_json_client::Client,

    account: Arc<Mutex<Account>>,
    priv_key: <<FilaSpec as Spec>::CryptoSpec as CryptoSpec>::PrivateKey,
}

impl Hub {
    pub fn new(
        rpc_url: String,
        account: Arc<Mutex<Account>>,
        priv_key: <<FilaSpec as Spec>::CryptoSpec as CryptoSpec>::PrivateKey,
    ) -> Result<Self> {
        let rpc_client = ClientBuilder::default().build()?;
        let ledger_client = sov_ledger_json_client::Client::new(&format!("{}/ledger", &rpc_url));
        let sequencer_client =
            sov_sequencer_json_client::Client::new(&format!("{}/sequencer", &rpc_url));
        let json_client =
            jsonrpsee::http_client::HttpClientBuilder::default().build(rpc_url.clone())?;

        Ok(Hub {
            rpc_url,
            rpc_client,
            json_client,
            ledger_client,
            sequencer_client,
            account,
            priv_key,
        })
    }

    pub async fn get_nonce(&self, credential_id: CredentialId) -> Result<u64> {
        let resp = self
            .rpc_client
            .get(format!(
                "{}/modules/nonces/state/nonces/items/{}",
                self.rpc_url, credential_id
            ))
            .send()
            .await?;

        let resp = resp.json::<ResponseObject<NonceResponse>>().await?;

        let nonce = resp
            .data
            .map(|data| data.value)
            .unwrap_or_default()
            .unwrap_or_default();

        Ok(nonce)
    }

    pub async fn get_events(&self, slot: u64) -> Result<()> {
        let events = self
            .ledger_client
            .get_slot_filtered_events(&IntOrHash::Variant0(slot), None)
            .await?;

        trace!(events = ?events);

        Ok(())
    }

    pub async fn last_slot(&self) -> Result<u64> {
        let slot = self.ledger_client.get_latest_slot(None).await?;

        trace!(fun = "last_slot", slot = ?slot);

        Ok(slot.data.number)
    }

    pub async fn pull_campaign(&self, campaign_id: u64) -> Result<Campaign<FilaSpec>> {
        let campaign: sov_modules_api::rest::utils::ResponseObject<Campaign<FilaSpec>> = self
            .json_client
            .request("core_getCampaign", rpc_params!(campaign_id))
            .await?;

        match campaign {
            ResponseObject {
                data: Some(campaign),
                ..
            } => Ok(campaign),
            ResponseObject {
                data: None, errors, ..
            } => Err(eyre!("got nothing back: {:?}", errors)),
        }
    }

    pub async fn pull_indexer(&self, credential_id: CredentialId) -> Result<Indexer<FilaSpec>> {
        let indexer: sov_modules_api::rest::utils::ResponseObject<Indexer<FilaSpec>> = self
            .json_client
            .request("core_getIndexer", rpc_params!(credential_id))
            .await?;

        match indexer {
            ResponseObject {
                data: Some(indexer),
                ..
            } => Ok(indexer),
            ResponseObject {
                data: None, errors, ..
            } => Err(eyre!("got nothing back: {:?}", errors)),
        }
    }

    pub async fn is_relayer(&self, credential_id: CredentialId) -> Result<bool> {
        // XXX: is there an endpoint to check?
        Ok(true)
    }

    pub async fn register_relay(&self, addr: <FilaSpec as Spec>::Address) -> Result<()> {
        let call = CallMessage::<FilaSpec>::RegisterRelayer { address: addr };
        let tx = self.sign_tx(borsh::to_vec(&call)?).await?;

        let body = serde_json::json!({ "body": tx });

        // XXX: this should probably go into its own function, that collects/batches tx and then
        //      submits the batch
        let res: serde_json::Value = self
            .json_client
            .request("sequencer_acceptTx", [body])
            .await
            .context("unable to submit tx")?;
        trace!("sequencer_acceptTx res: {:?}", res);

        let res: serde_json::Value = self
            .json_client
            .request("sequencer_publishBatch", tx.clone())
            .await
            .context("unable to publish batch")?;
        trace!("sequencer_publishBatch res: {:?}", res);

        Ok(())
    }

    pub async fn sign_tx(&self, msg: Vec<u8>) -> Result<Vec<u8>> {
        let chain_id = 0;
        let max_priority_fee_bips = PriorityFeeBips::from(100u64);
        let max_fee = 10000u64;

        let nonce: u64;
        {
            let acc = self.account.lock().await;
            nonce = acc.nonce;
        }

        let utx =
            UnsignedTransaction::new(msg, chain_id, max_priority_fee_bips, max_fee, nonce, None);

        let tx = Transaction::<FilaSpec>::new_signed_tx(&self.priv_key, utx);

        Ok(borsh::to_vec(&tx)?)
    }
}

pub async fn event_handler(ev: Event<FilaSpec>) -> Result<()> {
    match ev {
        Event::CampaignInitialized {
            campaign_id,
            campaigner,
            evictions,
        } => {
            // XXX: on campaign init we need to check for bonds on eth?
            info!(event_type = "CampaignInitialized", campaign_id = ?campaign_id, campaigner = ?campaigner, evictions = ?evictions);
        },
        Event::CampaignIndexing {
            campaign_id,
            indexer,
        } => {
            info!(event_type = "CampaignIndexing", campaign_id = ?campaign_id, indexer = ?indexer);
        },
        Event::SegmentPosted {
            campaign_id,
            indexer,
        } => {
            info!(event_type = "SegmentPosted", campaign_id = ?campaign_id, indexer = ?indexer);
        },
        Event::CriteriaProposed {
            campaign_id,
            proposer,
            proposal_id,
        } => {
            info!(event_type = "CriteriaProposed", campaign_id = ?campaign_id, proposer = ?proposer, proposal_id = ?proposal_id);
        },
        Event::CriteriaConfirmed {
            campaign_id,
            proposal_id,
        } => {
            info!(event_type = "CriteriaConfirmed", campaign_id = ?campaign_id, proposal_id = ?proposal_id);
        },
        Event::IndexerRegistered {
            addr,
            alias,
            sender,
        } => {
            // Relayer does not care
            info!(event_type = "IndexerRegistered", addr = ?addr, alias = ?alias, sender = ?sender)
        },
        Event::IndexerUnregistered { addr, sender } => {
            // Relayer does not care
            info!(event_type = "IndexerUnregistered", addr = ?addr, sender = ?sender)
        },
        Event::RelayerRegistered { addr, sender } => {
            // Relayer does not care
            info!(event_type = "RelayerRegistered", addr = ?addr, sender = ?sender)
        },
        Event::RelayerUnregistered { addr, sender } => {
            // XXX: relayer should shut down if it is unregistered?
            info!(event_type = "RelayerUnregistered", addr = ?addr, sender = ?sender)
        },
        Event::VotingPowerUpdated {
            addr,
            power,
            relayer,
        } => {
            // XXX: relayer cares?
            info!(event_type = "VotingPowerUpdated", addr = ?addr, power = ?power, relayer = ?relayer);
        },
    }
    Ok(())
}

pub async fn slot_watcher(rpc_url: String, lss: watch::Sender<u64>) -> Result<()> {
    trace!(fun = "slot_watcher", "entering");
    let ledger_client = sov_ledger_json_client::Client::new(&format!("{}/ledger", &rpc_url));
    let mut subscription = ledger_client.subscribe_slots().await?;

    while let Some(Ok(o)) = subscription.next().await {
        trace!(fun = "hub::slot_watcher", slot = ?o);
        lss.send(o.number)?;
    }

    Ok(())
}

#[derive(Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
pub struct Account {
    pub credential_id: CredentialId,
    pub nonce: u64,
}

// Each new slot, sync the hub account state
pub async fn account_watcher(
    rpc_url: String,
    credential_id: CredentialId,
    acc: Arc<Mutex<Account>>,
    mut lsr: watch::Receiver<u64>,
) -> Result<()> {
    trace!("hub account");

    let rpc_client = ClientBuilder::default().build()?;
    let nonce_url = format!(
        "{}/modules/nonces/state/nonces/items/{}",
        rpc_url, credential_id
    );

    loop {
        tokio::select! {
            Ok(()) = lsr.changed() => {
                let a = match rpc_client.get(nonce_url.clone()).send().await?.json::<ResponseObject<NonceResponse>>().await? {
                    ResponseObject { data: Some(NonceResponse { value: Some(nonce), .. }), .. } => Account { credential_id, nonce },
                    _ => Account { credential_id, nonce: 0 },
                };
                trace!(fun = "hub::account_watcher", credential_id = credential_id.to_string(), nonce = a.nonce);
                let mut c = acc.lock().await;
                *c = a;
            },
            else => break,
        }
    }
    Ok(())
}
