use std::sync::Arc;

use alloy::{
    network::EthereumWallet,
    primitives::Address,
    providers::ProviderBuilder,
    rpc::types::Block,
    sol,
    transports::http::{reqwest::Url, Client, Http},
};
use eyre::Result;
use tokio::sync::watch;
use tracing::trace;
use DelegateRegistry::DelegateRegistryInstance;

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    #[derive(Debug)]
    DelegateRegistry,
    "abi/DelegateRegistry.json"
);

// Codegen from ABI file to interact with the contract.
pub struct DelegateRegistrySync {
    // provider: DefaultFillProvider,
    instance: DelegateRegistryInstance<Http<Client>, super::eth::DefaultFillProvider>,
}

impl DelegateRegistrySync {
    pub fn new(addr: Address, rpc_url: Url, wallet: EthereumWallet) -> Self {
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(wallet)
            .on_http(rpc_url);
        DelegateRegistrySync {
            instance: DelegateRegistry::new(addr, provider),
        }
    }

    pub async fn pull_delegates(&self) -> Result<Vec<Address>> {
        let delegates = self.instance.allDelegates().call().await?;

        Ok(delegates._0)
    }

    // XXX(pm): would probably be best to just return the tx and then sign etc
    //          in a seperate module.
    pub async fn push_delegates(&self, addrs: Vec<Address>) -> Result<()> {
        let pending = self.instance.setDelegates(addrs).send().await?;
        trace!(who = "DelegateRegistry::push_delegates");
        Ok(())
    }

    pub async fn filter(&self, block: &Block) -> Result<()> {
        while let Some(tx) = block.transactions.txns().next() {
            trace!("{:?}", tx);
        }
        // block.
        Ok(())
    }
}

pub async fn listener(
    drs: Arc<DelegateRegistrySync>,
    mut lbr: watch::Receiver<Block>,
) -> Result<()> {
    loop {
        tokio::select! {
            Ok(()) = lbr.changed() => {
                let a = drs.pull_delegates().await?;
                let block = lbr.borrow();
                trace!(block_num = block.header.number, balance = ?a);
            },
            else => break,
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use alloy::providers::{Provider, ProviderBuilder, WalletProvider};
    use eyre::Result;

    use super::{DelegateRegistry, DelegateRegistrySync};

    #[tokio::test]
    async fn test_pull_delegates_empty() -> Result<()> {
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .on_anvil_with_wallet();

        let addr = provider.default_signer_address();
        let contract = DelegateRegistry::deploy(&provider, addr).await?;

        let fts = DelegateRegistrySync::new(
            *contract.address(),
            provider.client().transport().url().parse()?,
            provider.wallet().clone(),
        );

        assert_eq!(fts.pull_delegates().await?.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_push_delegates() -> Result<()> {
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .on_anvil_with_wallet();

        let addr = provider.default_signer_address();
        let contract = DelegateRegistry::deploy(&provider, addr).await?;

        let fts = DelegateRegistrySync::new(
            *contract.address(),
            provider.client().transport().url().parse()?,
            provider.wallet().clone(),
        );

        let addrs = provider.get_accounts().await?;
        fts.push_delegates(addrs.clone()).await?;

        assert_eq!(fts.pull_delegates().await?.len(), addrs.len());

        Ok(())
    }
}
