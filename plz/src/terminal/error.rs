#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// If this error is returned help is displayed.
    #[error("help invoked")]
    Help,
    /// If this error is returned usage is displayed.
    #[error("usage invoked")]
    Usage,
    /// Display an error with a hint.
    #[error("{err}")]
    Hint {
        err: eyre::Report,
        hint: &'static str,
    },
}
