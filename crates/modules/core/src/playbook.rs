use sov_bank::Coins;

use crate::crypto::Ed25519Signature;

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
    schemars(rename = "Playbook")
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
    schemars(rename = "Budget")
)]
pub struct Budget {
    pub fee: Coins,
    pub incentives: Coins,
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
    schemars(rename = "SegmentDescription")
)]
pub struct SegmentDescription {
    pub kind: SegmentKind,
    pub sources: Vec<String>,
    pub proof: SegmentProofMechanism,
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
    schemars(rename = "SegmentKind")
)]
pub enum SegmentKind {
    GithubTopNContributors(u16),
    GithubAllContributors,
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
    schemars(rename = "SegmentProofMechanism")
)]
pub enum SegmentProofMechanism {
    Ed25519Signature,
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
    schemars(rename = "ConversionDescription")
)]
pub struct ConversionDescription {
    pub kind: ConversionMechanism,
    pub proof: ConversionProofMechanism,
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
    schemars(rename = "ConversionMechanism")
)]
pub enum ConversionMechanism {
    Social(Auth),
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
    schemars(rename = "ConversionProofMechanism")
)]
pub enum ConversionProofMechanism {
    Ed25519Signature,
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
    schemars(rename = "ConversionProof")
)]
pub enum ConversionProof {
    Ed25519Signature(Ed25519Signature),
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
    schemars(rename = "Auth")
)]
pub enum Auth {
    Github,
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
    schemars(rename = "PayoutMechanism")
)]
pub enum PayoutMechanism {
    ProportionalPerConversion,
}
