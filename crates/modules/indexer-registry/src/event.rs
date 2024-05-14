use sov_modules_api::Spec;

/// IndexerRegistry Event
#[derive(
    Clone,
    Debug,
    PartialEq,
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum Event<S: Spec> {
    IndexerRegistered { addr: S::Address, alias: String },
    IndexerUnregistered { addr: S::Address },
}
