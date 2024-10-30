use sov_bank::Coins;

use crate::crypto::Ed25519Signature;

#[cfg_attr(
    feature = "native",
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet),
    schemars(rename = "Segment")
)]
#[derive(
    Clone,
    Debug,
    PartialEq,
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Deserialize,
    serde::Serialize,
    ts_rs::TS,
)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../../bindings/Segment.ts")]
pub struct Segment {
    pub data: SegmentData,
    pub proof: SegmentProof,
    pub retrieved_at: u128,
}

#[cfg_attr(
    feature = "native",
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet),
    schemars(rename = "SegmentData")
)]
#[derive(
    Clone,
    Debug,
    PartialEq,
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Deserialize,
    serde::Serialize,
    ts_rs::TS,
)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../../bindings/SegmentData.ts")]
pub enum SegmentData {
    Plain { allocations: Vec<(String, u64)> },
}

type GithubId = u64;

#[cfg_attr(
    feature = "native",
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet),
    schemars(rename = "GithubSegment")
)]
#[derive(
    Clone,
    Debug,
    PartialEq,
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Deserialize,
    serde::Serialize,
)]
pub struct GithubSegment {
    pub entries: Vec<(GithubId, Coins)>,
}

#[cfg_attr(
    feature = "native",
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet),
    schemars(rename = "SegmentProof")
)]
#[derive(
    Clone,
    Debug,
    PartialEq,
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Deserialize,
    serde::Serialize,
    ts_rs::TS,
)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../../bindings/SegmentProof.ts")]
pub enum SegmentProof {
    Ed25519Signature(Ed25519Signature),
}
