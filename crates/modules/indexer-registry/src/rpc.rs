use sov_modules_api::WorkingSet;

use super::IndexerRegistry;

#[derive(Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Indexer {
    pub addr: String,
    pub alias: String,
}

#[derive(Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct QueryIndexersResponse {
    pub indexers: Vec<Indexer>,
}

impl<S: sov_modules_api::Spec> IndexerRegistry<S> {
    /// Queries the state of the module.
    pub fn query_indexers(&self, working_set: &mut WorkingSet<S>) -> QueryIndexersResponse {
        let addrs = self.indexers.iter(working_set).collect::<Vec<_>>();

        QueryIndexersResponse {
            indexers: addrs
                .iter()
                .map(|addr| Indexer {
                    alias: self.aliases.get(&addr, working_set).unwrap_or_default(),
                    addr: addr.to_string(),
                })
                .collect::<Vec<_>>(),
        }
    }
}
