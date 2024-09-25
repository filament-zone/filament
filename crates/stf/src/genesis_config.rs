use std::path::{Path, PathBuf};

use filament_hub_core::CoreConfig;
use serde::de::DeserializeOwned;
use sov_accounts::AccountConfig;
use sov_attester_incentives::AttesterIncentivesConfig;
use sov_bank::BankConfig;
use sov_modules_api::{DaSpec, Spec};
use sov_modules_stf_blueprint::Runtime as RuntimeTrait;
use sov_prover_incentives::ProverIncentivesConfig;
use sov_sequencer_registry::SequencerConfig;

use super::GenesisConfig;
use crate::Runtime;

/// Paths to genesis files.
pub struct GenesisPaths {
    /// Accounts genesis path.
    pub accounts_genesis_path: PathBuf,
    /// Bank genesis path.
    pub bank_genesis_path: PathBuf,
    /// Sequencer Registry genesis path.
    pub sequencer_genesis_path: PathBuf,
    /// Attester Incentives genesis path.
    pub attester_incentives_genesis_path: PathBuf,
    /// Prover Incentives genesis path.
    pub prover_incentives_genesis_path: PathBuf,
    /// Hub Core genesis path.
    pub core_gensis_path: PathBuf,
}

impl GenesisPaths {
    /// Creates a new [`RuntimeTrait::GenesisConfig`] from the files contained in
    /// the given directory.
    ///
    /// Take a look at the contents of the `test_data` directory to see the
    /// expected files.
    pub fn from_dir(dir: impl AsRef<Path>) -> Self {
        Self {
            accounts_genesis_path: dir.as_ref().join("accounts.json"),
            bank_genesis_path: dir.as_ref().join("bank.json"),
            sequencer_genesis_path: dir.as_ref().join("sequencer_registry.json"),
            attester_incentives_genesis_path: dir.as_ref().join("attester_incentives.json"),
            prover_incentives_genesis_path: dir.as_ref().join("prover_incentives.json"),
            core_gensis_path: dir.as_ref().join("core.json"),
        }
    }
}

/// Creates a new [`GenesisConfig`] from the files contained in the given
/// directory.
pub fn create_genesis_config<S: Spec, Da: DaSpec>(
    genesis_paths: &GenesisPaths,
) -> anyhow::Result<<Runtime<S, Da> as RuntimeTrait<S, Da>>::GenesisConfig> {
    let accounts_config: AccountConfig<S> =
        read_genesis_json(&genesis_paths.accounts_genesis_path)?;
    let bank_config: BankConfig<S> = read_genesis_json(&genesis_paths.bank_genesis_path)?;
    let sequencer_registry_config: SequencerConfig<S, Da> =
        read_genesis_json(&genesis_paths.sequencer_genesis_path)?;
    let attester_incentives_config: AttesterIncentivesConfig<S> =
        read_genesis_json(&genesis_paths.attester_incentives_genesis_path)?;

    let prover_incentives_config: ProverIncentivesConfig<S> =
        read_genesis_json(&genesis_paths.prover_incentives_genesis_path)?;

    let nonces_config = ();

    let core_config: CoreConfig<S> = read_genesis_json(&genesis_paths.core_gensis_path)?;

    Ok(GenesisConfig::new(
        accounts_config,
        nonces_config,
        bank_config,
        sequencer_registry_config,
        attester_incentives_config,
        prover_incentives_config,
        core_config,
    ))
}

fn read_genesis_json<T: DeserializeOwned>(path: &Path) -> anyhow::Result<T> {
    let contents = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&contents)?)
}
