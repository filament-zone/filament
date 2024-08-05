use std::sync::Arc;

use eyre::{eyre, Context, Result};
use filament_hub_core::{campaign::Campaign, CallMessage, CoreRpcClient};
use jsonrpsee::{
    core::client::{ClientT, SubscriptionClientT},
    rpc_params,
    ws_client::WsClientBuilder as HubWsClientBuilder,
};
use sov_ledger_apis::rpc::client::RpcClient;
use sov_modules_api::{
    default_spec::DefaultSpec,
    transaction::{PriorityFeeBips, Transaction},
    CryptoSpec,
    PrivateKey,
    Spec,
};
use sov_risc0_adapter::Risc0Verifier;
use sov_rollup_interface::rpc::{BatchResponse, SlotResponse, TxResponse};
use tokio::sync::{watch, Mutex};
use tracing::debug;

pub type FilaSpec = DefaultSpec<Risc0Verifier, Risc0Verifier>;

#[derive(Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
pub enum AccResp {
    /// The account corresponding to the given public key exists.
    AccountExists {
        /// The address of the account,
        addr: String,
        /// The nonce of the account.
        nonce: u64,
    },
    /// The account corresponding to the given public key does not exist.
    AccountEmpty,
}

#[derive(Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize, Clone)]
pub struct Account {
    pub pkh: [u8; 32],
    pub nonce: u64,
}

pub struct Hub {
    pub account: Arc<Mutex<Account>>,
    pub priv_key: <<FilaSpec as Spec>::CryptoSpec as CryptoSpec>::PrivateKey,
    // why
    // pub hub_client: Arc<Box<dyn RpcClient<SlotResponse<u32, u32>, BatchResponse<u32, u32>,
    // TxResponse<u32>>>>,
    pub endpoint: String,
}

impl Hub {
    pub fn new(
        endpoint: String,
        account: Arc<Mutex<Account>>,
        priv_key: <<FilaSpec as Spec>::CryptoSpec as CryptoSpec>::PrivateKey,
    ) -> Self {
        Hub {
            endpoint,
            account,
            priv_key,
            // outpost_campaigns: HashMap::new(),
        }
    }

    pub async fn create_campaign(&self, call: CallMessage<FilaSpec>) -> Result<()> {
        let hws = rpc_client(self.endpoint.clone()).await;
 
        debug!("call: {:?}", borsh::to_vec(&call)?);
        // let tx = self.sign_tx(serde_json::to_vec(&call)?).await?;
        let tx = self.sign_tx(borsh::to_vec(&call)?).await?;
        debug!("tx: {:?}", tx);

        let body = serde_json::json!({ "body": tx});
        let res: serde_json::Value = hws
            .request("sequencer_acceptTx", [body])
            .await
            .context("unable to submit tx")?;
        debug!("sequencer_acceptTx res: {:?}", res);

        let res: serde_json::Value = hws
            .request("sequencer_publishBatch", tx.clone())
            .await
            .context("unable to publish batch")?;
        debug!("sequencer_publishBatch res: {:?}", res);

        Ok(())
    }

    pub async fn pull_campaign(&self, campaign_id: u64) -> Result<Campaign<FilaSpec>> {
        debug!("pull_hub_campaign({:})", campaign_id);

        let hws = rpc_client(self.endpoint.clone()).await;
        let campaign: Campaign<FilaSpec> =
            if let Some(campaign) = hws.rpc_get_campaign(campaign_id).await? {
                campaign
            } else {
                return Err(eyre!("campaign {} not found", campaign_id));
            };

        Ok(campaign)
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

        let tx = Transaction::<FilaSpec>::new_signed_tx(
            &self.priv_key,
            msg,
            chain_id,
            max_priority_fee_bips,
            max_fee,
            None,
            nonce,
        );

        Ok(borsh::to_vec(&tx)?)
    }
}

// Each new slot, sync the hub account state
pub async fn account_watcher(
    ws_client: Arc<
        impl RpcClient<SlotResponse<u32, u32>, BatchResponse<u32, u32>, TxResponse<u32>>
            + SubscriptionClientT,
    >,
    pkh: [u8; 32],
    acc: Arc<Mutex<Account>>,
    mut lsr: watch::Receiver<u64>,
) -> Result<()> {
    debug!("hub account");
    loop {
        tokio::select! {
            Ok(()) = lsr.changed() => {
                let a = match ws_client.request("accounts_getAccount", rpc_params!(pkh.to_vec())).await? {
                    AccResp::AccountExists { addr: _, nonce } => Account { pkh, nonce },
                    _ => Account { pkh, nonce: 0 },
                };
                let mut c = acc.lock().await;
                *c = a;
            },
            else => break,
        }
    }
    Ok(())
}

// Push out new slot notifications
pub async fn latest_slot(url: String, lss: watch::Sender<u64>) -> Result<()> {
    debug!("latest_slot");
    let hws = rpc_client(url).await;
    let mut sub = hws.subscribe_slots().await?;

    while let Some(Ok(o)) = sub.next().await {
        debug!("new slot: {:}", o);
        lss.send(o)?;
    }

    Ok(())
}

pub async fn rpc_client(
    url: String,
) -> Arc<
    impl RpcClient<SlotResponse<u32, u32>, BatchResponse<u32, u32>, TxResponse<u32>>
        + SubscriptionClientT,
> {
    Arc::new(HubWsClientBuilder::new().build(url).await.unwrap())
}
