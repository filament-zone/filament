use jsonrpsee::core::RpcResult;
use sov_modules_api::{macros::rpc_gen, Spec, WorkingSet};

use crate::{Campaign, Campaigns, Segment};

#[rpc_gen(client, server, namespace = "campaign")]
impl<S: Spec> Campaigns<S> {
    #[rpc_method(name = "getCampaign")]
    pub fn rpc_get_campaign(
        &self,
        id: u64,
        working_set: &mut WorkingSet<S>,
    ) -> RpcResult<Option<Campaign<S>>> {
        Ok(self.get_campaign(id, working_set))
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
