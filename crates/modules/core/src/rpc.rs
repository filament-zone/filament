use jsonrpsee::core::RpcResult;
use sov_modules_api::{macros::rpc_gen, Spec, WorkingSet};

use crate::{Campaign, Core, Indexer, Segment};

#[rpc_gen(client, server, namespace = "campaign")]
impl<S: Spec> Core<S> {
    #[rpc_method(name = "getCampaign")]
    pub fn rpc_get_campaign(
        &self,
        id: u64,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<Option<Campaign<S>>> {
        Ok(self.get_campaign(id, working_set))
    }

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

    #[rpc_method(name = "getSegment")]
    pub fn rpc_get_segment(
        &self,
        id: u64,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<Option<Segment>> {
        Ok(self.get_segment(id, working_set))
    }
}
