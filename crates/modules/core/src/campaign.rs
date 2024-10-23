use sov_modules_api::Spec;

use crate::{
    criteria::Criteria,
    delegate::{Delegate, Eviction},
};

pub const SEVICTION_COST: u64 = 1;
pub const MAX_EVICTIONS: u64 = 3;

#[cfg_attr(
    feature = "native",
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet),
    schemars(bound = "S: ::sov_modules_api::Spec", rename = "Campaign")
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
)]
#[serde(bound = "S::Address: serde::Serialize + serde::de::DeserializeOwned")]
pub struct Campaign<S: Spec> {
    pub campaigner: S::Address,
    pub phase: Phase,

    pub title: String,
    pub description: String,

    pub criteria: Criteria,

    pub evictions: Vec<Eviction<S>>,
    // TODO(xla): Rework into commitments in follow-up.
    pub delegates: Vec<Delegate<S>>,

    pub indexer: Option<S::Address>,
}

#[cfg_attr(
    feature = "native",
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet),
    schemars(rename = "Phase")
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
)]
pub enum Phase {
    Init,
    Criteria,
    Publish,
    Indexing,
    Distribution,
    Settle,
    Settled,
    Canceled,
    Rejected,
}
