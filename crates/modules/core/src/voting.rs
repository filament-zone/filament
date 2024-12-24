pub type Power = u64;

#[cfg_attr(
    feature = "native",
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet)
)]
#[derive(
    Clone,
    Debug,
    PartialEq,
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Serialize,
    serde::Deserialize,
    ts_rs::TS,
)]
#[ts(export_to = "../../../../bindings/VoteOption.ts")]
pub enum VoteOption {
    Yes { weights: Vec<u64> },
    No,
}
