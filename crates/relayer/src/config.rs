use crate::error::Error;
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub ethereum_rpc_url: String,
    pub hub_url: String,
    pub delegate_registry_address: String,
    pub polling_interval_seconds: u64,
    pub database_path: String,
    pub hub_private_key: String, // Handle securely!  Consider environment variable.
    pub max_retries: u32,
    pub retry_backoff_seconds: u64,
    pub genesis_block: u64, // Add genesis block
    pub batch_size: u64,    // Add polling batch size
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, Error> {
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }
}
