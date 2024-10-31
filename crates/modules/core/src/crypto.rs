#[cfg_attr(
    feature = "native",
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet),
    schemars(rename = "Ed25519Signature")
)]
#[derive(
    Clone,
    Debug,
    Eq,
    PartialEq,
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Deserialize,
    serde::Serialize,
    ts_rs::TS,
)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../../bindings/Ed25519Signature.ts")]
pub struct Ed25519Signature {
    // FIXME(xla): Deriving JsonSchena breaks serde_with, therefore we have to explicitly assign
    // both functions here.
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
