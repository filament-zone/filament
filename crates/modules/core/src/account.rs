#[cfg_attr(
    feature = "native",
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet),
    schemars(rename = "Account")
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
#[ts(export, concrete(S = DefaultSpec<MockZkVerifier, MockZkVerifier, Zk>))]
#[ts(export_to = "../../../../bindings/Account.ts")]
pub struct Account {
    pub nonce: u64,
}
