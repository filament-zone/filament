use sov_mock_zkvm::MockZkVerifier;
use sov_modules_api::{default_spec::DefaultSpec, execution_mode::Zk, Spec};

#[cfg_attr(
    feature = "native",
    derive(schemars::JsonSchema),
    derive(sov_modules_api::macros::UniversalWallet),
    schemars(bound = "S: ::sov_modules_api::Spec", rename = "Delegate")
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
#[ts(export_to = "../../../../bindings/Delegate.ts")]
pub struct Delegate<S: Spec> {
    #[ts(type = "string")]
    pub address: S::Address,
    pub alias: String,
}

pub type Eviction<S> = <S as Spec>::Address;
