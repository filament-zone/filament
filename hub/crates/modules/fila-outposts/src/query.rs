use jsonrpsee::core::RpcResult;
use sov_modules_api::{macros::rpc_gen, Context, StateMapAccessor, WorkingSet};

use crate::OutpostRegistry;

/// Response for `getOwner` method
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct OutpostResponse {
    /// Outpost chain id.
    pub chain_id: String,
}

#[rpc_gen(client, server, namespace = "outpost_registry")]
impl<C> OutpostRegistry<C>
where
    C: Context,
{
    /// Get the outpost for a given chain_id.
    #[rpc_method(name = "getOutpost")]
    pub fn get_outpost(
        &self,
        chain_id: String,
        working_set: &mut WorkingSet<C>,
    ) -> RpcResult<Option<OutpostResponse>> {
        let res = self
            .outposts
            .get(&chain_id, working_set)
            .map(|_| OutpostResponse { chain_id });
        Ok(res)
    }
}
