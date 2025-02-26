use anyhow::Context;
use clap::Parser;
use filament_relayer::cli::{run_cli, Cli, CliCommand};
use filament_relayer::config::Config;
use filament_relayer::database::Database;
use filament_relayer::ethereum::EthereumClient;
use filament_relayer::hub::HubClient;
use filament_relayer::relayer::Relayer;
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
use web3::transports::Http;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    // Load configuration
    let config_path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());
    info!(config_path = %config_path, "Loading configuration");
    let config = Config::load(PathBuf::from(config_path).as_path())
        .context("Failed to load configuration")?;

    // Create Ethereum client
    info!(
        ethereum_rpc_url = %config.ethereum_rpc_url,
        "Connecting to Ethereum node",
    );
    let transport =
        Http::new(&config.ethereum_rpc_url).context("Failed to create Ethereum transport")?;
    let web3 = web3::Web3::new(transport);
    let ethereum_client = EthereumClient::new(web3, config.delegate_registry_address.clone())
        .context("Failed to create Ethereum client")?;

    // Create Hub client
    info!(hub_url = %config.hub_url, "Creating Hub client");
    let hub_client = HubClient::new(config.hub_url.clone());

    // Create Database
    info!(database_path = %config.database_path, "Opening database");
    let database =
        Database::new(&config.database_path).context("Failed to create/open database")?;

    // Create Relayer instance
    let relayer = Relayer::new(config, ethereum_client, hub_client, database);

    // Run CLI (starts the relayer if the `Start` command is given)
    run_cli(relayer).context("Failed during CLI execution")?;

    Ok(())
}
