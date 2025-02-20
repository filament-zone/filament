use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error), // Use #[from] for automatic conversion
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error), // For file system operations

    #[error("TOML parsing error: {0}")]
    TomlError(#[from] toml::de::Error), // For TOML deserialization

    #[error("Serialization error: {0}")]
    SerializationError(#[from] bincode::Error), // For serialization/deserialization

    #[error("Sled database error: {0}")]
    SledError(#[from] sled::Error), // For sled database operations

    #[error("Web3 error: {0}")]
    Web3Error(#[from] web3::Error),

    #[error("Ethereum RPC error: {0}")]
    EthereumRpcError(web3::Error), // Keep this as a named variant

    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("Hub error: {0}")]
    HubError(String), //Keep this as a named variant

    #[error("Hub transaction confirmation failed")] //Keep as named variant
    HubConfirmationFailed,

    #[error("Other error: {0}")] // Keep this as a named variant for string errors
    Other(String),
}
