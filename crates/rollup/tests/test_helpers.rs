use std::{net::SocketAddr, path::Path, str::FromStr};

use filament_hub_rollup::mock_rollup::MockRollup;
use filament_hub_stf::genesis_config::GenesisPaths;
use k256::ecdsa::SigningKey;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::Sha256;
use sov_cli::wallet_state::PrivateKeyAndAddress;
use sov_kernels::basic::{BasicKernelGenesisConfig, BasicKernelGenesisPaths};
use sov_mock_da::MockDaConfig;
use sov_modules_api::{Address, Spec};
use sov_modules_rollup_blueprint::FullNodeBlueprint;
use sov_sequencer::FairBatchBuilderConfig;
use sov_stf_runner::{
    processes::RollupProverConfig,
    HttpServerConfig,
    ProofManagerConfig,
    RollupConfig,
    RunnerConfig,
    SequencerConfig,
    StorageConfig,
};
use tokio::sync::oneshot;

const PROVER_ADDRESS: &str = "sov1pv9skzctpv9skzctpv9skzctpv9skzctpv9skzctpv9skzctpv9stup8tx";

pub async fn start_rollup(
    rpc_reporting_channel: oneshot::Sender<SocketAddr>,
    rest_reporting_channel: oneshot::Sender<SocketAddr>,
    rt_genesis_paths: GenesisPaths,
    kernel_genesis_paths: BasicKernelGenesisPaths,
    rollup_prover_config: RollupProverConfig,
    da_config: MockDaConfig,
) {
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();
    let sequencer_address = da_config.sender_address;

    let rollup_config = RollupConfig {
        storage: StorageConfig {
            path: temp_path.to_path_buf(),
        },
        runner: RunnerConfig {
            genesis_height: 0,
            da_polling_interval_ms: 1000,
            rpc_config: HttpServerConfig {
                bind_host: "127.0.0.1".into(),
                bind_port: 0,
            },
            axum_config: HttpServerConfig {
                bind_host: "127.0.0.1".into(),
                bind_port: 0,
            },
            concurrent_sync_tasks: Some(1),
        },
        da: da_config,
        proof_manager: ProofManagerConfig {
            aggregated_proof_block_jump: 1,
            prover_address: Address::<Sha256>::from_str(PROVER_ADDRESS)
                .expect("Prover address is not valid"),
        },
        sequencer: SequencerConfig {
            max_allowed_blocks_behind: 5,
            batch_builder: FairBatchBuilderConfig {
                mempool_max_txs_count: None,
                max_batch_size_bytes: None,
                sequencer_address,
            },
        },
    };

    let mock_demo_rollup = MockRollup::default();

    let kernel_genesis = BasicKernelGenesisConfig {
        chain_state: serde_json::from_str(
            &std::fs::read_to_string(&kernel_genesis_paths.chain_state)
                .expect("Failed to read chain_state genesis config"),
        )
        .expect("Failed to parse chain_state genesis config"),
    };

    let rollup = mock_demo_rollup
        .create_new_rollup(
            &rt_genesis_paths,
            kernel_genesis,
            rollup_config,
            Some(rollup_prover_config),
        )
        .await
        .unwrap();

    rollup
        .run_and_report_addr(Some(rpc_reporting_channel), Some(rest_reporting_channel))
        .await
        .unwrap();

    // Close the tempdir explicitly to ensure that rustc doesn't see that it's unused and drop it
    // unexpectedly
    temp_dir.close().unwrap();
}

pub fn read_private_keys<S: Spec>(suffix: &str) -> PrivateKeyAndAddress<S> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    let private_keys_dir = Path::new(&manifest_dir).join("../../test-data/keys");

    let data = std::fs::read_to_string(private_keys_dir.join(suffix))
        .expect("Unable to read file to string");

    let key_and_address: PrivateKeyAndAddress<S> =
        serde_json::from_str(&data).unwrap_or_else(|_| {
            panic!("Unable to convert data {} to PrivateKeyAndAddress", &data);
        });

    assert!(
        key_and_address.is_matching_to_default(),
        "Inconsistent key data"
    );

    key_and_address
}

/// A struct representing an ETH signing key and associated address.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound = "S::Address: Serialize + DeserializeOwned")]
pub struct EthSigningKeyAndAddres<S: sov_modules_api::Spec> {
    /// Signing key of the address.
    pub signing_key: String,
    /// Address associated from the private key.
    pub address: S::Address,
}

pub fn read_eth_key<S: Spec>(suffix: &str) -> anyhow::Result<(SigningKey, S::Address)> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    let private_keys_dir = Path::new(&manifest_dir).join("../../test-data/eth");

    let data = std::fs::read_to_string(private_keys_dir.join(suffix))
        .expect("Unable to read file to string");

    let parsed: EthSigningKeyAndAddres<S> = serde_json::from_str(&data).unwrap_or_else(|_| {
        panic!("Unable to convert data {} to PrivateKeyAndAddress", &data);
    });

    let signing_key = SigningKey::from_bytes(hex::decode(parsed.signing_key)?.as_slice().into())?;

    Ok((signing_key, parsed.address))
}
