use std::collections::HashMap;

use sov_modules_api::Spec;

type DatasetId = String;
type Field = String;
type Predicate = String;

#[cfg_attr(
    feature = "native",
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet),
    schemars(rename = "Criterion")
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
pub struct Criterion {
    pub dataset_id: DatasetId,
    pub parameters: HashMap<Field, Predicate>,
}

pub type Criteria = Vec<Criterion>;

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
    schemars(rename = "CriteriaProposal")
)]
pub struct CriteriaProposal<S: Spec> {
    pub campaign_id: u64,
    pub proposer: S::Address,
    pub criteria: Criteria,
}
