use jsonrpsee::core::RpcResult;
use sov_modules_api::{
    macros::rpc_gen,
    prelude::{
        axum::{routing::get, Router},
        serde_yaml,
        UnwrapInfallible as _,
    },
    rest::{
        utils::{errors, ApiResult, Path},
        ApiState,
        HasCustomRestApi,
    },
    ApiStateAccessor,
    Spec,
};

use crate::{criteria::CriteriaProposal, Campaign, Core, Indexer, Segment};

#[rpc_gen(client, server, namespace = "core")]
impl<S: Spec> Core<S> {
    #[rpc_method(name = "getCampaign")]
    pub fn rpc_get_campaign(
        &self,
        id: u64,
        state: &mut ApiStateAccessor<S>,
    ) -> RpcResult<Option<Campaign<S>>> {
        Ok(self.get_campaign(id, state).unwrap_infallible())
    }

    #[rpc_method(name = "getCriteriaProposal")]
    pub fn rpc_get_criteria_proposal(
        &self,
        campaign_id: u64,
        proposal_id: u64,
        state: &mut ApiStateAccessor<S>,
    ) -> RpcResult<Option<CriteriaProposal<S>>> {
        Ok(self
            .get_criteria_proposal(campaign_id, proposal_id, state)
            .unwrap_infallible())
    }

    /// Returns the list of currently registered indexers.
    #[rpc_method(name = "getIndexer")]
    pub fn rpc_get_indexer(
        &self,
        addr: S::Address,
        state: &mut ApiStateAccessor<S>,
    ) -> RpcResult<Option<Indexer<S>>> {
        Ok(self.get_indexer(addr, state).unwrap_infallible())
    }

    /// Returns the list of currently registered indexers.
    #[rpc_method(name = "getIndexers")]
    pub fn rpc_get_indexers(&self, state: &mut ApiStateAccessor<S>) -> RpcResult<Vec<Indexer<S>>> {
        Ok(self.get_indexers(state).unwrap_infallible())
    }

    #[rpc_method(name = "getSegment")]
    pub fn rpc_get_segment(
        &self,
        id: u64,
        state: &mut ApiStateAccessor<S>,
    ) -> RpcResult<Option<Segment>> {
        Ok(self.get_segment(id, state).unwrap_infallible())
    }
}

// Axum routes.
impl<S: Spec> Core<S> {
    async fn route_get_campaign(
        state: ApiState<Self, S>,
        Path(campaign_id): Path<u64>,
    ) -> ApiResult<Campaign<S>> {
        let campaign = state
            .get_campaign(campaign_id, &mut state.api_state_accessor())
            .unwrap_infallible()
            .ok_or_else(|| errors::not_found_404("Campaign", campaign_id))?;
        Ok(campaign.into())
    }
}

impl<S: Spec> HasCustomRestApi for Core<S> {
    type Spec = S;

    fn custom_rest_api(&self, state: ApiState<Self, Self::Spec>) -> Router<()> {
        Router::new()
            .route("/campaigns/:campaignId", get(Self::route_get_campaign))
            .with_state(state)
    }

    fn custom_openapi_spec(&self) -> Option<sov_modules_api::rest::OpenApi> {
        let open_api =
            serde_yaml::from_str(include_str!("../openapi-v3.yaml")).expect("Invalid OpenAPI spec");
        Some(open_api)
    }
}
