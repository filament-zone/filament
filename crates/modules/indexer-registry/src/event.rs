/// Template Event
#[derive(
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Serialize,
    serde::Deserialize,
    Debug,
    PartialEq,
    Clone,
)]
pub enum Event {
    IndexerRegistered { addr: String, alias: String },
    IndexerUnregistered { addr: String },
}
