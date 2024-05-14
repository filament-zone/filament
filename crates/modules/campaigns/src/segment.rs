use sov_bank::Coins;

use crate::crypto::Ed25519Signature;

#[derive(
    Clone,
    Debug,
    PartialEq,
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Deserialize,
    serde::Serialize,
)]
pub struct Segment {
    pub data: SegmentData,
    pub proof: SegmentProof,
    pub retrieved_at: u128,
}

#[derive(
    Clone,
    Debug,
    PartialEq,
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Deserialize,
    serde::Serialize,
)]
pub enum SegmentData {
    GithubSegment(GithubSegment),
}

type GithubId = u64;

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

#[derive(
    Clone,
    Debug,
    PartialEq,
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Deserialize,
    serde::Serialize,
)]
pub enum SegmentProof {
    Ed25519Signature(Ed25519Signature),
}
