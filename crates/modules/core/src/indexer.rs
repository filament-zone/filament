use sov_modules_api::Spec;

pub type Alias = String;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(
    feature = "native",
    derive(schemars::JsonSchema),
    schemars(bound = "S: ::sov_modules_api::Spec", rename = "Indexer")
)]
#[serde(bound = "S::Address: serde::Serialize + serde::de::DeserializeOwned")]
pub struct Indexer<S: Spec> {
    pub addr: S::Address,
    pub alias: Alias,
}
