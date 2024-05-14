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
pub struct Playbook {
    pub budget: Budget,
    pub segment_description: SegmentDescription,
    pub conversion_description: ConversionDescription,
    pub payout: PayoutMechanism,
    pub ends_at: u128,
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
pub struct Budget {
    pub fee: Coins,
    pub incentives: Coins,
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
pub struct SegmentDescription {
    pub kind: SegmentKind,
    pub sources: Vec<String>,
    pub proof: SegmentProofMechanism,
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
pub enum SegmentKind {
    GithubTopNContributors(u16),
    GithubAllContributors,
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
pub enum SegmentProofMechanism {
    Ed25519Signature,
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
pub struct ConversionDescription {
    pub kind: ConversionMechanism,
    pub proof: ConversionProofMechanism,
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
pub enum ConversionMechanism {
    Social(Auth),
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
pub enum ConversionProofMechanism {
    Ed25519Signature,
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
pub enum ConversionProof {
    Ed25519Signature(Ed25519Signature),
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
pub enum Auth {
    Github,
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
pub enum PayoutMechanism {
    ProportionalPerConversion,
}
