use std::collections::HashMap;

use sov_modules_api::Spec;

type DatasetId = String;
type Field = String;
type Predicate = String;

#[cfg_attr(
    feature = "native",
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet),
    schemars(rename = "CriterionContract")
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
#[ts(export_to = "../../../../bindings/CriterionContract.ts")]
pub enum Contract {
    Ethereum { address: String },
}

#[cfg_attr(
    feature = "native",
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet),
    schemars(rename = "CriterionCategory")
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
#[ts(export_to = "../../../../bindings/CriterionCategory.ts")]
pub enum CriterionCategory {
    Balance,
    Defi,
    Gaming,
    Governance,
    Nft,
}

#[cfg_attr(
    feature = "native",
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet),
    schemars(rename = "CriterionType")
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
#[ts(export_to = "../../../../bindings/CriterionType.ts")]
pub enum CriterionType {
    LiquidityProvider { contract: Contract },
    TvlByContract { contract: Contract },
    VolumeByContract { contract: Contract },
}

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
    ts_rs::TS,
)]
#[ts(export_to = "../../../../bindings/Criterion.ts")]
pub struct Criterion {
    pub name: String,
    pub category: CriterionCategory,
    pub parameters: HashMap<Field, Predicate>,
    pub weight: u64,
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
