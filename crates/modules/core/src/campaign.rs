use sov_modules_api::Spec;

use crate::playbook::Playbook;

pub type ChainId = String;

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
    schemars(bound = "S: ::sov_modules_api::Spec", rename = "Campaign")
)]
#[serde(bound = "S::Address: serde::Serialize + serde::de::DeserializeOwned")]
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
#[cfg_attr(
    feature = "native",
    derive(schemars::JsonSchema),
    schemars(rename = "Status")
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
