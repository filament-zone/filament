#[derive(
    Clone,
    Debug,
    PartialEq,
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Deserialize,
    serde::Serialize,
)]
pub struct Ed25519Signature {
    #[serde(with = "serde_bytes")]
    pub pk: [u8; 32],
    #[serde(with = "serde_bytes")]
    pub sig: [u8; 64],
}
