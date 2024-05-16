//! The Rollup entrypoint.
//!
//! On a high level, the rollup node receives serialized call messages from the DA layer and
//! executes them as atomic transactions. Upon reception, the message has to be deserialized and
//! forwarded to an appropriate module.
//!
//! The module-specific logic is implemented by module creators, but all the glue code responsible
//! for message deserialization/forwarding is handled by a rollup `runtime`.
//!
//! In order to define the runtime we need to specify all the modules supported by our rollup (see
//! the `Runtime` struct bellow)
//!
//! The `Runtime` together with associated interfaces (`Genesis`, `DispatchCall`, `MessageCodec`)
//! and derive macros defines:
//! - how the rollup modules are wired up together.
//! - how the state of the rollup is initialized.
//! - how messages are dispatched to appropriate modules.
//!
//! Runtime lifecycle:
//!
//! 1. Initialization: When a rollup is deployed for the first time, it needs to set its genesis
//!    state. The `#[derive(Genesis)` macro will generate `Runtime::genesis(config)` method which
//!    returns `Storage` with the initialized state.
//!
//! 2. Calls:       The `Module` interface defines a `call` method which accepts a module-defined
//!    type and triggers the specific `module logic.` In general, the point of a call is to change
//!    the module state, but if the call throws an error, no module specific state is updated (the
//!    transaction is reverted).
//!
//! `#[derive(MessageCodec)` adds deserialization capabilities to the `Runtime` (implements
//! `decode_call` method). `Runtime::decode_call` accepts serialized call message and returns a type
//! that implements the `DispatchCall` trait.  The `DispatchCall` implementation (derived by a
//! macro) forwards the message to the appropriate module and executes its `call` method.

#![allow(unused_doc_comments)]

#[cfg(feature = "native")]
use filament_hub_core::{CoreRpcImpl, CoreRpcServer};
#[cfg(feature = "native")]
use sov_accounts::{AccountsRpcImpl, AccountsRpcServer};
#[cfg(feature = "native")]
use sov_bank::{BankRpcImpl, BankRpcServer};
#[cfg(feature = "native")]
use sov_modules_api::macros::{expose_rpc, CliWallet};
use sov_modules_api::{DispatchCall, Event, Genesis, MessageCodec, Spec};
#[cfg(feature = "native")]
use sov_prover_incentives::{ProverIncentivesRpcImpl, ProverIncentivesRpcServer};
use sov_rollup_interface::da::DaSpec;
#[cfg(feature = "native")]
use sov_sequencer_registry::{SequencerRegistryRpcImpl, SequencerRegistryRpcServer};

#[cfg(feature = "native")]
use crate::genesis::GenesisPaths;

/// The hub-stf runtime.
#[cfg_attr(feature = "native", derive(CliWallet), expose_rpc)]
#[derive(Default, Genesis, DispatchCall, Event, MessageCodec)]
#[serialization(
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Serialize,
    serde::Deserialize
)]
pub struct Runtime<S: Spec, Da: DaSpec> {
    /// The Accounts module.
    pub accounts: sov_accounts::Accounts<S>,
    /// The Bank module.
    pub bank: sov_bank::Bank<S>,
    /// The Prover Incentives module.
    pub prover_incentives: sov_prover_incentives::ProverIncentives<S, Da>,
    /// The Sequencer Registry module.
    pub sequencer_registry: sov_sequencer_registry::SequencerRegistry<S, Da>,

    /// The hub core module.
    pub core: filament_hub_core::Core<S>,
}

impl<S, Da> sov_modules_stf_blueprint::Runtime<S, Da> for Runtime<S, Da>
where
    S: Spec,
    Da: DaSpec,
{
    type GenesisConfig = GenesisConfig<S, Da>;
    #[cfg(feature = "native")]
    type GenesisPaths = GenesisPaths;

    #[cfg(feature = "native")]
    fn rpc_methods(storage: tokio::sync::watch::Receiver<S::Storage>) -> jsonrpsee::RpcModule<()> {
        get_rpc_methods::<S, Da>(storage)
    }

    #[cfg(feature = "native")]
    fn genesis_config(
        genesis_paths: &Self::GenesisPaths,
    ) -> Result<Self::GenesisConfig, anyhow::Error> {
        crate::genesis::create_genesis_config(genesis_paths)
    }
}
