use std::str::FromStr;

use anyhow::Context as _;
use filament_hub_stf::runtime::EthereumToRollupAddressConverter;
use sov_ethereum::{EthRpcConfig, EthereumAuthenticator, GasPriceOracleConfig};
use sov_modules_api::Spec;
use sov_rollup_interface::node::da::DaService;
use tokio::sync::watch;

// register ethereum methods.
pub(crate) fn register_ethereum<
    S: Spec,
    Da: DaService,
    RT: EthereumAuthenticator<S> + Send + Sync + 'static,
>(
    da_service: Da,
    storage: watch::Receiver<<S as Spec>::Storage>,
    methods: &mut jsonrpsee::RpcModule<()>,
) -> anyhow::Result<()>
where
    EthereumToRollupAddressConverter: TryInto<S::Address>,
{
    let eth_rpc_config = {
        let eth_signer = eth_dev_signer();
        EthRpcConfig {
            min_blob_size: Some(1),
            eth_signer,
            gas_price_oracle_config: GasPriceOracleConfig::default(),
        }
    };

    let ethereum_rpc =
        sov_ethereum::get_ethereum_rpc::<S, Da, RT>(da_service, eth_rpc_config, storage);
    methods
        .merge(ethereum_rpc)
        .context("Failed to merge Ethereum RPC modules")
}

// TODO: https://github.com/Sovereign-Labs/sovereign-sdk-wip/issues/387
fn eth_dev_signer() -> sov_ethereum::DevSigner {
    sov_ethereum::DevSigner::new(vec![secp256k1::SecretKey::from_str(
        "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80",
    )
    .unwrap()])
}
