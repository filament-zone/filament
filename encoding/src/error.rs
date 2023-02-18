#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("encoding failed: {0}")]
    Bcs(#[from] bcs::Error),

    #[error(transparent)]
    Bech32(#[from] bech32::Error),

    #[error(transparent)]
    Ed25519(#[from] pulzaar_crypto::Ed25519Error),

    #[error("unexpected bech32 prefix")]
    Bech32UnexpectedPrefix,

    #[error("unexpected bech32 variant")]
    Bech32UnexpectedVariant,

    #[error("invalid input to parse type")]
    Bech32Conversion,
}
