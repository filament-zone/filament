//! While the `GenesisConfig` type for `Rollup` is generated from the underlying runtime through a
//! macro, specific module configurations are obtained from files. This code is responsible for the
//! logic that transforms module genesis data into Rollup genesis data.

use std::{
    convert::AsRef,
    path::{Path, PathBuf},
};

use filament_hub_core::CoreConfig;
use serde::de::DeserializeOwned;
pub use sov_accounts::{AccountConfig, AccountData};
use sov_attester_incentives::AttesterIncentivesConfig;
pub use sov_bank::{BankConfig, Coins, TokenConfig};
pub use sov_chain_state::ChainStateConfig;
pub use sov_evm::EvmConfig;
use sov_modules_api::prelude::*;
use sov_modules_stf_blueprint::Runtime as RuntimeTrait;
pub use sov_nft::NonFungibleTokenConfig;
use sov_prover_incentives::ProverIncentivesConfig;
pub use sov_sequencer_registry::SequencerConfig;
pub use sov_state::config::Config as StorageConfig;
pub use sov_value_setter::ValueSetterConfig;

use crate::runtime::Runtime;
/// Creates config for a rollup with some default settings, the config is used in demos and
/// tests.
use crate::runtime::{EthereumToRollupAddressConverter, GenesisConfig};

/// Paths pointing to genesis files.
pub struct GenesisPaths {
    /// Bank genesis path.
    pub bank_genesis_path: PathBuf,
    /// Sequencer Registry genesis path.
    pub sequencer_genesis_path: PathBuf,
    /// Value Setter genesis path.
    pub value_setter_genesis_path: PathBuf,
    /// Accounts genesis path.
    pub accounts_genesis_path: PathBuf,
    /// Attester Incentives genesis path.
    pub attester_incentives_genesis_path: PathBuf,
    /// Prover Incentives genesis path.
    pub prover_incentives_genesis_path: PathBuf,
    /// NFT genesis path.
    pub nft_path: PathBuf,
    /// EVM genesis path.
    pub evm_genesis_path: PathBuf,

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
            bank_genesis_path: dir.as_ref().join("bank.json"),
            sequencer_genesis_path: dir.as_ref().join("sequencer_registry.json"),
            value_setter_genesis_path: dir.as_ref().join("value_setter.json"),
            accounts_genesis_path: dir.as_ref().join("accounts.json"),
            prover_incentives_genesis_path: dir.as_ref().join("prover_incentives.json"),
            attester_incentives_genesis_path: dir.as_ref().join("attester_incentives.json"),
            nft_path: dir.as_ref().join("nft.json"),
            evm_genesis_path: dir.as_ref().join("evm.json"),
            core_genesis_path: dir.as_ref().join("core.json"),
        }
    }
}

/// Creates a new [`RuntimeTrait::GenesisConfig`] from the files contained in
/// the given directory.
pub fn create_genesis_config<S: Spec, Da: DaSpec>(
    genesis_paths: &GenesisPaths,
) -> anyhow::Result<<Runtime<S, Da> as RuntimeTrait<S, Da>>::GenesisConfig>
where
    EthereumToRollupAddressConverter: TryInto<S::Address>,
{
    let bank_config: BankConfig<S> = read_genesis_json(&genesis_paths.bank_genesis_path)?;

    let sequencer_registry_config: SequencerConfig<S, Da> =
        read_genesis_json(&genesis_paths.sequencer_genesis_path)?;

    let value_setter_config: ValueSetterConfig<S> =
        read_genesis_json(&genesis_paths.value_setter_genesis_path)?;

    let attester_incentives_config: AttesterIncentivesConfig<S> =
        read_genesis_json(&genesis_paths.attester_incentives_genesis_path)?;

    let prover_incentives_config: ProverIncentivesConfig<S> =
        read_genesis_json(&genesis_paths.prover_incentives_genesis_path)?;

    let accounts_config: AccountConfig<S> =
        read_genesis_json(&genesis_paths.accounts_genesis_path)?;

    let nonces_config = ();

    let nft_config: NonFungibleTokenConfig = read_genesis_json(&genesis_paths.nft_path)?;

    let evm_config: EvmConfig = read_genesis_json(&genesis_paths.evm_genesis_path)?;

    let core_config: CoreConfig<S> = read_genesis_json(&genesis_paths.core_genesis_path)?;

    Ok(GenesisConfig::new(
        bank_config,
        sequencer_registry_config,
        value_setter_config,
        attester_incentives_config,
        prover_incentives_config,
        accounts_config,
        nonces_config,
        nft_config,
        evm_config,
        core_config,
    ))
}

fn read_genesis_json<T: DeserializeOwned>(path: &Path) -> anyhow::Result<T> {
    let contents = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&contents)?)
}
