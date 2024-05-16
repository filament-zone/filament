//! While the `GenesisConfig` type for `Rollup` is generated from the underlying runtime through a
//! macro, specific module configurations are obtained from files. This code is responsible for the
//! logic that transforms module genesis data into Rollup genesis data.

use std::{
    convert::AsRef,
    path::{Path, PathBuf},
};

pub use filament_hub_core::CoreConfig;
pub use sov_accounts::{AccountConfig, AccountData};
pub use sov_bank::{BankConfig, Coins, TokenConfig};
pub use sov_chain_state::ChainStateConfig;
use sov_modules_api::Spec;
use sov_modules_stf_blueprint::Runtime as RuntimeTrait;
use sov_prover_incentives::ProverIncentivesConfig;
use sov_rollup_interface::da::DaSpec;
pub use sov_sequencer_registry::SequencerConfig;
pub use sov_state::config::Config as StorageConfig;
use sov_stf_runner::read_json_file;

/// Creates config for a rollup with some default settings, the config is used in demos and
/// tests.
use crate::runtime::GenesisConfig;
use crate::runtime::Runtime;

/// Paths pointing to genesis files.
pub struct GenesisPaths {
    /// Accounts genesis path.
    pub accounts_genesis_path: PathBuf,
    /// Bank genesis path.
    pub bank_genesis_path: PathBuf,
    /// Prover Incentives genesis path.
    pub prover_incentives_genesis_path: PathBuf,
    /// Sequencer Registry genesis path.
    pub sequencer_genesis_path: PathBuf,

    /// Core genesis path.
    pub core_genesis_path: PathBuf,
}

impl GenesisPaths {
    /// Creates a new [`GenesisPaths`] from the files contained in the given
    /// directory.
    ///
    /// Take a look at the contents of the `test-data` directory to see the
    /// expected files.
    pub fn from_dir(dir: impl AsRef<Path>) -> Self {
        Self {
            accounts_genesis_path: dir.as_ref().join("accounts.json"),
            bank_genesis_path: dir.as_ref().join("bank.json"),
            prover_incentives_genesis_path: dir.as_ref().join("prover_incentives.json"),
            sequencer_genesis_path: dir.as_ref().join("sequencer_registry.json"),

            core_genesis_path: dir.as_ref().join("core.json"),
        }
    }
}

/// Creates a new [`GenesisConfig`] from the files contained in the given
/// directory.
pub fn create_genesis_config<S: Spec, Da: DaSpec>(
    genesis_paths: &GenesisPaths,
) -> anyhow::Result<<Runtime<S, Da> as RuntimeTrait<S, Da>>::GenesisConfig> {
    let accounts_config: AccountConfig<S> = read_json_file(&genesis_paths.accounts_genesis_path)?;

    let bank_config: BankConfig<S> = read_json_file(&genesis_paths.bank_genesis_path)?;

    let prover_incentives_config: ProverIncentivesConfig<S> =
        read_json_file(&genesis_paths.prover_incentives_genesis_path)?;

    let sequencer_registry_config: SequencerConfig<S, Da> =
        read_json_file(&genesis_paths.sequencer_genesis_path)?;

    let core_config: CoreConfig<S> = read_json_file(&genesis_paths.core_genesis_path)?;

    Ok(GenesisConfig::new(
        accounts_config,
        bank_config,
        prover_incentives_config,
        sequencer_registry_config,
        core_config,
    ))
}
