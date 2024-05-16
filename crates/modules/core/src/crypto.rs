#[derive(
    Clone,
    Debug,
    Eq,
    PartialEq,
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Deserialize,
    serde::Serialize,
)]
#[cfg_attr(
    feature = "native",
    derive(schemars::JsonSchema),
    schemars(rename = "Ed25519Signature")
)]
pub struct Ed25519Signature {
    // FIXME(xla): Deriving JsonSchena breaks serde with, therefore we have to be explicitly assign
    // both functins here.
    //
    // https://github.com/GREsau/schemars/issues/89
    #[serde(deserialize_with = "serde_bytes::deserialize")]
    #[serde(serialize_with = "serde_bytes::serialize")]
    pub pk: [u8; 32],
    // FIXME(xla): JsonSchema doesn't have an implementation for a byte array of size 64.
    // #[serde(deserialize_with = "serde_bytes::deserialize")]
    // #[serde(serialize_with = "serde_bytes::serialize")]
    // pub sig: [u8; 64],
}
