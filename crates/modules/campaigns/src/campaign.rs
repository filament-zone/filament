use sov_modules_api::Spec;

use crate::playbook::Playbook;

pub type ChainId = String;

#[derive(
    Clone,
    Debug,
    PartialEq,
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Deserialize,
    serde::Serialize,
)]
pub struct Campaign<S: Spec> {
    pub status: Status,

    pub origin: ChainId,
    pub origin_id: u64,

    pub indexer: S::Address,
    pub attester: S::Address,
    pub playbook: Playbook,
}

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
pub enum Status {
    Created,
    Funded,
    Indexing,
    Attesting,
    Finished,
    Canceled,
    Failed(String),
}
