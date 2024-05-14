use jsonrpsee::core::RpcResult;
use sov_modules_api::{macros::rpc_gen, Spec, WorkingSet};

use crate::{indexer::Indexer, IndexerRegistry};

#[rpc_gen(client, server, namespace = "indexer_registry")]
impl<S: Spec> IndexerRegistry<S> {
    /// Returns the list of currently registered indexers.
    #[rpc_method(name = "getIndexer")]
    pub fn rpc_get_indexer(
        &self,
        addr: S::Address,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<Option<Indexer<S>>> {
        Ok(self.get_indexer(addr, working_set))
    }

    /// Returns the list of currently registered indexers.
    #[rpc_method(name = "getIndexers")]
    pub fn rpc_get_indexers(&self, working_set: &mut WorkingSet<S>) -> RpcResult<Vec<Indexer<S>>> {
        Ok(self.get_indexers(working_set))
    }
}
