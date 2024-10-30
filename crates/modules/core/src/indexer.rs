use sov_mock_zkvm::MockZkVerifier;
use sov_modules_api::{default_spec::DefaultSpec, execution_mode::Zk, Spec};

pub type Alias = String;

#[cfg_attr(
    feature = "native",
    derive(schemars::JsonSchema),
    schemars(bound = "S: ::sov_modules_api::Spec", rename = "Indexer")
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
#[ts(export_to = "../../../../bindings/Indexer.ts")]
pub struct Indexer<S: Spec> {
    #[ts(type = "string")]
    pub addr: S::Address,
    pub alias: Alias,
}
