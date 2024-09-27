use std::sync::Arc;

use alloy::{
    consensus::Account,
    network::{Ethereum, EthereumWallet},
    primitives::{Address, U256},
    providers::{
        fillers::{
            BlobGasFiller,
            ChainIdFiller,
            FillProvider,
            GasFiller,
            JoinFill,
            NonceFiller,
            WalletFiller,
        },
        Identity,
        Provider,
        ProviderBuilder,
        RootProvider,
        WsConnect,
    },
    rpc::types::Block,
    transports::http::Http,
};
use eyre::Result;
use futures::{lock::Mutex, StreamExt};
use reqwest::Client;
use tokio::sync::watch;
use tracing::trace;

pub type DefaultFillProvider = FillProvider<
    JoinFill<
        JoinFill<
            Identity,
            JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>,
        >,
        WalletFiller<EthereumWallet>,
    >,
    RootProvider<Http<Client>>,
    Http<Client>,
    Ethereum,
>;

pub async fn block_watcher(
    ws_client: WsConnect,
    lbs: watch::Sender<Block>,
    start: u64,
    stop: u64,
) -> Result<()> {
    trace!("entering block_watcher");
    let block_provider = ProviderBuilder::new().on_ws(ws_client).await?;
    let block_sub = block_provider.subscribe_blocks().await?;

    let mut stream = block_sub.into_stream();
    while let Some(block) = stream.next().await {
        trace!("block received: {:?}", block);
        lbs.send(block)?;
    }

    Ok(())
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct EthAccount {
    pub address: Address,
    pub nonce: u64,
    pub balance: U256,
}

pub async fn account_watcher(
    ws_client: WsConnect,
    mut lbr: watch::Receiver<Block>,
    addr: Address,
    acc: Arc<Mutex<EthAccount>>,
) -> Result<()> {
    trace!("entering block_watcher");
    let provider = ProviderBuilder::new().on_ws(ws_client).await?;

    loop {
        tokio::select! {
            Ok(()) = lbr.changed() => {
                let a = match provider.get_account(addr).await? {
                    Account { nonce, balance, .. } => EthAccount { address: addr, nonce, balance },
                };
                trace!("account state after new block: {:?}", a);
                let mut c = acc.lock().await;
                *c = a;
            },
            else => break,
        }
    }

    Ok(())
}
