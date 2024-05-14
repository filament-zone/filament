use std::path::PathBuf;

use anyhow::Context as _;
use clap::Parser;
use filament_hub_rollup::{initialize_logging, CelestiaDemoRollup, MockDemoRollup};
use filament_hub_stf::genesis::GenesisPaths;
use sov_kernels::basic::{BasicKernelGenesisConfig, BasicKernelGenesisPaths};
use sov_mock_da::MockDaConfig;
use sov_modules_rollup_blueprint::{Rollup, RollupBlueprint};
use sov_stf_runner::{from_toml_path, RollupConfig, RollupProverConfig};
use tracing::debug;

#[cfg(test)]
mod test_rpc;

/// Main demo runner. Initializes a DA chain, and starts a demo-rollup using the provided.
/// If you're trying to sign or submit transactions to the rollup, the `sov-cli` binary
/// is the one you want. You can run it `cargo run --bin sov-cli`.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The data layer type.
    #[arg(long, default_value = "mock")]
    da_layer: SupportedDaLayer,

    /// The path to the rollup config.
    #[arg(long, default_value = "config/mock_rollup_config.toml")]
    rollup_config_path: String,

    /// The path to the genesis configs.
    #[arg(long, default_value = "../test-data/genesis/demo/mock")]
    genesis_config_dir: PathBuf,

    /// Listen address for Prometheus exporter.
    #[arg(long, default_value = "127.0.0.1:9845")]
    prometheus_exporter_bind: String,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum SupportedDaLayer {
    Celestia,
    Mock,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    initialize_logging();
    let args = Args::parse();

    prometheus_exporter::start(args.prometheus_exporter_bind.parse()?)
        .expect("Failed to start prometheus exporter");

    let rollup_config_path = args.rollup_config_path.as_str();

    let prover_config = if option_env!("CI").is_some() {
        Some(RollupProverConfig::Execute)
    } else {
        option_env!("SOV_PROVER_MODE").map(|prover| match prover {
            "simulate" => RollupProverConfig::Simulate,
            "execute" => RollupProverConfig::Execute,
            "prove" => RollupProverConfig::Prove,
            "skip" => RollupProverConfig::Skip,
            _ => {
                tracing::warn!(
                    prover_mode = prover,
                    "Unknown sov prover mode, using 'Skip' default"
                );
                RollupProverConfig::Skip
            },
        })
    };

    tracing::info!(?prover_config, "Running demo rollup with prover config");

    match args.da_layer {
        SupportedDaLayer::Mock => {
            let chain_state_config = args.genesis_config_dir.join("chain_state.json");
            let rollup = new_rollup_with_mock_da(
                &GenesisPaths::from_dir(&args.genesis_config_dir),
                &BasicKernelGenesisPaths {
                    chain_state: chain_state_config,
                },
                rollup_config_path,
                prover_config,
            )
            .await?;
            rollup.run().await
        },
        SupportedDaLayer::Celestia => {
            let chain_state_config = args.genesis_config_dir.join("chain_state.json");
            let rollup = new_rollup_with_celestia_da(
                &GenesisPaths::from_dir(&args.genesis_config_dir),
                &BasicKernelGenesisPaths {
                    chain_state: chain_state_config,
                },
                rollup_config_path,
                prover_config,
            )
            .await?;
            rollup.run().await
        },
    }
}

async fn new_rollup_with_celestia_da(
    rt_genesis_paths: &GenesisPaths,
    kernel_genesis_paths: &BasicKernelGenesisPaths,
    rollup_config_path: &str,
    prover_config: Option<RollupProverConfig>,
) -> Result<Rollup<CelestiaDemoRollup>, anyhow::Error> {
    debug!(config_path = rollup_config_path, "Starting Celestia rollup");

    let rollup_config: RollupConfig<sov_celestia_adapter::CelestiaConfig> =
        from_toml_path(rollup_config_path).context("Failed to read rollup configuration")?;

    let kernel_genesis = BasicKernelGenesisConfig {
        chain_state: serde_json::from_str(
            &std::fs::read_to_string(&kernel_genesis_paths.chain_state).with_context(|| {
                format!(
                    "Failed to read chain state from {}",
                    kernel_genesis_paths.chain_state.display()
                )
            })?,
        )?,
    };

    let mock_rollup = CelestiaDemoRollup {};
    mock_rollup
        .create_new_rollup(
            rt_genesis_paths,
            kernel_genesis,
            rollup_config,
            prover_config,
        )
        .await
}

async fn new_rollup_with_mock_da(
    rt_genesis_paths: &GenesisPaths,
    kernel_genesis_paths: &BasicKernelGenesisPaths,
    rollup_config_path: &str,
    prover_config: Option<RollupProverConfig>,
) -> Result<Rollup<MockDemoRollup>, anyhow::Error> {
    debug!(config_path = rollup_config_path, "Starting mock rollup");

    let rollup_config: RollupConfig<MockDaConfig> = from_toml_path(rollup_config_path)
        .with_context(|| {
            format!(
                "Failed to read rollup configuration from {}",
                rollup_config_path
            )
        })?;

    let kernel_genesis = BasicKernelGenesisConfig {
        chain_state: serde_json::from_str(
            &std::fs::read_to_string(&kernel_genesis_paths.chain_state).with_context(|| {
                format!(
                    "Failed to read chain state from {}",
                    kernel_genesis_paths.chain_state.display()
                )
            })?,
        )?,
    };

    let mock_rollup = MockDemoRollup {};
    mock_rollup
        .create_new_rollup(
            rt_genesis_paths,
            kernel_genesis,
            rollup_config,
            prover_config,
        )
        .await
}
