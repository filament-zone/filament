use sov_modules_api::Spec;

pub type Alias = String;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Indexer<S: Spec> {
    pub addr: S::Address,
    pub alias: Alias,
}
