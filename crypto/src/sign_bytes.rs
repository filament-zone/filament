/// Behaviour to produce the bytes expected to be signs for the type in question.
pub trait SignBytes {
    /// Returns the bytes to be signed with ['SigningKey`].
    fn sign_bytes(&self) -> eyre::Result<Vec<u8>>;
}
