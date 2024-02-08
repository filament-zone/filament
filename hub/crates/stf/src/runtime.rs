#![allow(unused_doc_comments)]

use sov_modules_api::macros::DefaultRuntime;
#[cfg(feature = "native")]
use sov_modules_api::Spec;
use sov_modules_api::{Context, DaSpec, DispatchCall, Genesis, MessageCodec};

#[cfg(feature = "native")]
pub use sov_accounts::{AccountsRpcImpl, AccountsRpcServer};
#[cfg(feature = "native")]
pub use sov_bank::{BankRpcImpl, BankRpcServer};
#[cfg(feature = "native")]
pub use sov_sequencer_registry::{SequencerRegistryRpcImpl, SequencerRegistryRpcServer};

#[cfg(feature = "native")]
pub use fila_outposts::{OutpostRegistryRpcImpl, OutpostRegistryRpcServer};

#[cfg(feature = "native")]
use crate::genesis_config::GenesisPaths;

#[cfg_attr(
    feature = "native",
    derive(sov_modules_api::macros::CliWallet),
    sov_modules_api::macros::expose_rpc
)]
#[derive(Genesis, DispatchCall, MessageCodec, DefaultRuntime)]
#[serialization(borsh::BorshDeserialize, borsh::BorshSerialize)]
#[cfg_attr(feature = "serde", serialization(serde::Serialize, serde::Deserialize))]
pub struct Runtime<C: Context, Da: DaSpec> {
    /// The `accounts` module is responsible for managing user accounts and their nonces
    pub accounts: sov_accounts::Accounts<C>,
    /// The bank module is responsible for minting, transferring, and burning tokens
    pub bank: sov_bank::Bank<C>,
    /// The sequencer registry module is responsible for authorizing sequencers to rollup transactions.
    pub sequencer_registry: sov_sequencer_registry::SequencerRegistry<C, Da>,

    pub outpost_registry: fila_outposts::OutpostRegistry<C>,
}

impl<C, Da> sov_modules_stf_blueprint::Runtime<C, Da> for Runtime<C, Da>
where
    C: Context,
    Da: DaSpec,
{
    type GenesisConfig = GenesisConfig<C, Da>;

    #[cfg(feature = "native")]
    type GenesisPaths = GenesisPaths;

    #[cfg(feature = "native")]
    fn rpc_methods(storage: <C as Spec>::Storage) -> jsonrpsee::RpcModule<()> {
        get_rpc_methods::<C, Da>(storage.clone())
    }

    #[cfg(feature = "native")]
    fn genesis_config(
        genesis_paths: &Self::GenesisPaths,
    ) -> Result<Self::GenesisConfig, anyhow::Error> {
        crate::genesis_config::get_genesis_config(genesis_paths)
    }
}
