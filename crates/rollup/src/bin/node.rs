//! This binary runs the rollup full node.

use std::{env, str::FromStr};

use anyhow::Context;
use clap::Parser;
#[cfg(feature = "celestia_da")]
use filament_hub_rollup::celestia_rollup::CelestiaRollup;
#[cfg(feature = "mock_da")]
use filament_hub_rollup::mock_rollup::MockRollup;
use filament_hub_stf::genesis_config::GenesisPaths;
use sha2::Sha256;
#[cfg(feature = "celestia_da")]
use sov_celestia_adapter::verifier::CelestiaSpec;
#[cfg(feature = "celestia_da")]
use sov_celestia_adapter::CelestiaConfig;
use sov_kernels::basic::{BasicKernelGenesisConfig, BasicKernelGenesisPaths};
#[cfg(feature = "mock_da")]
use sov_mock_da::MockDaConfig;
#[cfg(feature = "mock_da")]
use sov_mock_da::MockDaSpec;
use sov_modules_api::Address;
use sov_modules_rollup_blueprint::{FullNodeBlueprint, Rollup};
use sov_rollup_interface::execution_mode::Native;
use sov_sequencer::FairBatchBuilderConfig;
use sov_stf_runner::{from_toml_path, processes::RollupProverConfig, RollupConfig};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

#[cfg(all(feature = "mock_da", feature = "celestia_da"))]
compile_error!("Both mock_da and celestia_da are enabled, but only one should be.");

#[cfg(all(not(feature = "mock_da"), not(feature = "celestia_da")))]
compile_error!("Neither mock_da and celestia_da are enabled, but only one should be.");

// config and genesis for mock da
#[cfg(all(feature = "mock_da", not(feature = "celestia_da")))]
const DEFAULT_CONFIG_PATH: &str = "config/rollup_config.toml";
#[cfg(all(feature = "mock_da", not(feature = "celestia_da")))]
const DEFAULT_GENESIS_PATH: &str = "test-data/genesis/mock/";
#[cfg(all(feature = "mock_da", not(feature = "celestia_da")))]
const DEFAULT_KERNEL_GENESIS_PATH: &str = "test-data/genesis/mock/chain_state.json";

// config and genesis for local docker celestia
#[cfg(all(feature = "celestia_da", not(feature = "mock_da")))]
const DEFAULT_CONFIG_PATH: &str = "config/celestia_rollup_config.toml";
#[cfg(all(feature = "celestia_da", not(feature = "mock_da")))]
const DEFAULT_GENESIS_PATH: &str = "test-data/genesis/celestia/";
#[cfg(all(feature = "celestia_da", not(feature = "mock_da")))]
const DEFAULT_KERNEL_GENESIS_PATH: &str = "test-data/genesis/celestia/chain_state.json";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The path to the rollup config.
    #[arg(long, default_value = DEFAULT_CONFIG_PATH)]
    rollup_config_path: String,

    /// The path to the genesis config.
    #[arg(long, default_value = DEFAULT_GENESIS_PATH)]
    genesis_paths: String,
    /// The path to the kernel genesis config.
    #[arg(long, default_value = DEFAULT_KERNEL_GENESIS_PATH)]
    kernel_genesis_paths: String,

    /// The optional path to the log file.
    #[arg(long, default_value = None)]
    log_dir: Option<String>,

    /// The optional path to the log file.
    #[arg(long, default_value_t = 9845)]
    metrics: u64,
}

fn init_logging(log_dir: Option<String>) -> Option<WorkerGuard> {
    let stdout_layer = fmt::layer().with_writer(std::io::stdout);
    let filter_layer = EnvFilter::from_str(
        &env::var("RUST_LOG")
            .unwrap_or_else(|_| "debug,hyper=info,jmt=info,risc0_zkvm=info,reqwest=info,tower_http=info,jsonrpsee-client=info,jsonrpsee-server=info,sqlx=warn,tiny_http=warn,risc0_circuit_rv32im=info".to_string()))
            .unwrap();

    let subscriber = tracing_subscriber::registry()
        .with(stdout_layer)
        .with(filter_layer);

    if let Some(path) = log_dir {
        let file_appender = tracing_appender::rolling::daily(path, "rollup.log");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
        subscriber
            .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
            .init();
        Some(guard)
    } else {
        subscriber.init();
        None
    }
}

#[tokio::main]
// Not returning result here, so error could be logged properly.
async fn main() {
    let args = Args::parse();

    let guard = init_logging(args.log_dir);
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        tracing_panic::panic_hook(panic_info);
        prev_hook(panic_info);
    }));

    let metrics_port = args.metrics;
    let address = format!("127.0.0.1:{}", metrics_port);
    prometheus_exporter::start(address.parse().unwrap())
        .expect("Could not start prometheus server");

    let rollup_config_path = args.rollup_config_path.as_str();

    let genesis_paths = args.genesis_paths.as_str();
    let kernel_genesis_paths = args.kernel_genesis_paths.as_str();

    let prover_config = parse_prover_config().expect("Malformed prover_config");
    tracing::info!(?prover_config, "Running demo rollup with prover config");

    let rollup = new_rollup(
        &GenesisPaths::from_dir(genesis_paths),
        &BasicKernelGenesisPaths {
            chain_state: kernel_genesis_paths.into(),
        },
        rollup_config_path,
        prover_config,
    )
    .await
    .expect("Couldn't start rollup");
    rollup.run().await.expect("Couldn't run rollup");
    drop(guard);
}

#[allow(clippy::manual_inspect)]
fn parse_prover_config() -> anyhow::Result<Option<RollupProverConfig>> {
    if let Some(value) = option_env!("SOV_PROVER_MODE") {
        let config = std::str::FromStr::from_str(value).map_err(|error| {
            tracing::error!(value, ?error, "Unknown `SOV_PROVER_MODE` value; aborting");
            error
        })?;
        Ok(Some(config))
    } else {
        Ok(None)
    }
}

#[cfg(all(feature = "mock_da", not(feature = "celestia_da")))]
async fn new_rollup(
    rt_genesis_paths: &GenesisPaths,
    kernel_genesis_paths: &BasicKernelGenesisPaths,
    rollup_config_path: &str,
    prover_config: Option<RollupProverConfig>,
) -> Result<Rollup<MockRollup<Native>, Native>, anyhow::Error> {
    tracing::info!("Starting mock rollup with config {}", rollup_config_path);

    let rollup_config: RollupConfig<
        Address<Sha256>,
        MockDaConfig,
        FairBatchBuilderConfig<MockDaSpec>,
    > = from_toml_path(rollup_config_path).with_context(|| {
        format!(
            "Failed to read rollup configuration from {}",
            rollup_config_path
        )
    })?;

    let mock_rollup = MockRollup::default();

    let kernel_genesis = BasicKernelGenesisConfig {
        chain_state: serde_json::from_str(
            &std::fs::read_to_string(&kernel_genesis_paths.chain_state)
                .context("Failed to read chain state")?,
        )?,
    };

    mock_rollup
        .create_new_rollup(
            rt_genesis_paths,
            kernel_genesis,
            rollup_config,
            prover_config,
        )
        .await
}

#[cfg(all(feature = "celestia_da", not(feature = "mock_da")))]
async fn new_rollup(
    rt_genesis_paths: &GenesisPaths,
    kernel_genesis_paths: &BasicKernelGenesisPaths,
    rollup_config_path: &str,
    prover_config: Option<RollupProverConfig>,
) -> Result<Rollup<CelestiaRollup<Native>, Native>, anyhow::Error> {
    tracing::info!(
        "Starting Celestia rollup with config {}",
        rollup_config_path
    );

    let rollup_config: RollupConfig<
        Address<Sha256>,
        CelestiaConfig,
        FairBatchBuilderConfig<CelestiaSpec>,
    > = from_toml_path(rollup_config_path).with_context(|| {
        format!(
            "Failed to read rollup configuration from {}",
            rollup_config_path
        )
    })?;

    let kernel_genesis = BasicKernelGenesisConfig {
        chain_state: serde_json::from_str(
            &std::fs::read_to_string(&kernel_genesis_paths.chain_state)
                .context("Failed to read chain state")?,
        )?,
    };

    let celestia_rollup = CelestiaRollup::default();
    celestia_rollup
        .create_new_rollup(
            rt_genesis_paths,
            kernel_genesis,
            rollup_config,
            prover_config,
        )
        .await
}
