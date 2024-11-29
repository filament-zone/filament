use sov_mock_zkvm::MockZkVerifier;
use sov_modules_api::{default_spec::DefaultSpec, execution_mode::Zk, Spec};

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
    ts_rs::TS,
)]
#[serde(bound = "S::Address: serde::Serialize + serde::de::DeserializeOwned")]
#[ts(export, concrete(S = DefaultSpec<MockZkVerifier, MockZkVerifier, Zk>))]
#[ts(export_to = "../../../../bindings/Campaign.ts")]
pub struct Campaign<S: Spec> {
    #[ts(type = "string")]
    pub campaigner: S::Address,
    pub phase: Phase,

    pub title: String,
    pub description: String,

    pub criteria: Criteria,

    #[ts(type = "Array<string>")]
    pub evictions: Vec<Eviction<S>>,
    // TODO(xla): Rework into commitments in follow-up.
    #[ts(type = "Array<string>")]
    pub delegates: Vec<Delegate<S>>,

    #[ts(type = "string | null")]
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
    ts_rs::TS,
)]
#[ts(export_to = "../../../../bindings/Phase.ts")]
pub enum Phase {
    Draft,
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
