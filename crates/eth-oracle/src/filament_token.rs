use std::sync::Arc;

use alloy::{
    network::EthereumWallet,
    primitives::{address, Address, U256},
    providers::ProviderBuilder,
    rpc::types::Block,
    sol,
    transports::http::{reqwest::Url, Client, Http},
};
use eyre::Result;
use tokio::sync::watch;
use tracing::trace;
use FilamentToken::FilamentTokenInstance;

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    #[derive(Debug)]
    FilamentToken,
    "abi/FilamentToken.json"
);

// Codegen from ABI file to interact with the contract.
pub struct FilamentTokenSync {
    // provider: DefaultFillProvider,
    instance: FilamentTokenInstance<Http<Client>, super::eth::DefaultFillProvider>,
}

impl FilamentTokenSync {
    pub fn new(addr: Address, rpc_url: Url, wallet: EthereumWallet) -> Self {
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .wallet(wallet)
            .on_http(rpc_url);
        FilamentTokenSync {
            instance: FilamentToken::new(addr, provider),
        }
    }

    pub async fn pull_balance(&self, addr: Address) -> Result<U256> {
        let bal = self.instance.balanceOf(addr).call().await?;
        trace!(contract_name = "FilamentToken", contract_addr = ?self.instance.address(), who = ?addr, balance = ?bal._0);
        Ok(bal._0)
    }
}

pub async fn listener(fts: Arc<FilamentTokenSync>, mut lbr: watch::Receiver<Block>) -> Result<()> {
    let target = address!("41653c7d61609D856f29355E404F310Ec4142Cfb");

    loop {
        tokio::select! {
            Ok(()) = lbr.changed() => {
                let a = fts.pull_balance(target).await?;
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
    use alloy::{
        primitives::utils::parse_ether,
        providers::{Provider, ProviderBuilder, WalletProvider},
    };
    use eyre::Result;

    use super::{FilamentToken, FilamentTokenSync};

    #[tokio::test]
    async fn test_pull_balance() -> Result<()> {
        let provider = ProviderBuilder::new()
            .with_recommended_fillers()
            .on_anvil_with_wallet();

        let bal = parse_ether("10")?;
        let addr = provider.default_signer_address();
        let contract = FilamentToken::deploy(&provider, addr, bal).await?;

        let fts = FilamentTokenSync::new(
            *contract.address(),
            provider.client().transport().url().parse()?,
            provider.wallet().clone(),
        );

        assert_eq!(fts.pull_balance(addr).await?, bal);

        Ok(())
    }
}
