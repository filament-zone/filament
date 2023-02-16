#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("encoding failed: {0}")]
    Bcs(#[from] bcs::Error),
}
