use filament_chain::Address;

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("sender unauthorized")]
    SenderUnauthorized,
    #[error("asset '{0}' not supported")]
    AssetNotSupported(String),

    #[error("insufficient funds for account: {0}")]
    InsufficientFunds(Address),

    #[error("asset balance overflow")]
    BalanceOverflow,
    #[error("asset balanced underflow")]
    BalanceUnderflow,
}
