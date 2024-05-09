use sov_modules_api::Spec;

/// IndexerRegistry Event
#[derive(
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Serialize,
    serde::Deserialize,
    Debug,
    PartialEq,
    Clone,
)]
pub enum Event<S: Spec> {
    IndexerRegistered { addr: S::Address, alias: String },
    IndexerUnregistered { addr: S::Address },
}
