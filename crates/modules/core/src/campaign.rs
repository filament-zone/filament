use sov_bank::Coins;
use sov_modules_api::Spec;

use crate::{
    criteria::Criteria,
    delegate::{Delegate, Eviction},
    playbook::Budget,
};

pub const EVICTION_COST: u64 = 100;
pub const MAX_EVICTIONS: u64 = 3;

pub type ChainId = String;

#[cfg_attr(
    feature = "native",
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet),
    schemars(rename = "Payment")
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
pub struct Payment {
    coins: Coins,
}

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

    pub criteria: Criteria,
    pub budget: Budget,
    pub payments: Vec<Payment>,

    pub proposed_delegates: Vec<Delegate<S>>,
    pub evictions: Vec<Eviction<S>>,
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
}
