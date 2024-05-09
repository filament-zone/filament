use jsonrpsee::core::RpcResult;
use sov_modules_api::{macros::rpc_gen, WorkingSet};

use super::IndexerRegistry;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Indexer {
    pub addr: String,
    pub alias: String,
}

#[cfg_attr(
    feature = "native",
    derive(Clone, serde::Deserialize, serde::Serialize)
)]
#[derive(Debug, Eq, PartialEq)]
pub struct IndexersResponse {
    pub indexers: Vec<Indexer>,
}

#[rpc_gen(client, server, namespace = "indexer")]
impl<S: sov_modules_api::Spec> IndexerRegistry<S> {
    /// Returns the list of currently registered indexers.
    #[rpc_method(name = "getIndexers")]
    pub fn get_indexers(&self, working_set: &mut WorkingSet<S>) -> RpcResult<IndexersResponse> {
        let addrs = self.indexers.iter(working_set).collect::<Vec<_>>();

        Ok(IndexersResponse {
            indexers: addrs
                .iter()
                .map(|addr| Indexer {
                    alias: self.aliases.get(&addr, working_set).unwrap_or_default(),
                    addr: addr.to_string(),
                })
                .collect::<Vec<_>>(),
        })
    }
}
