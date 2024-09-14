use jsonrpsee::core::RpcResult;
use sov_modules_api::{
    macros::rpc_gen,
    prelude::{
        axum::{routing::get, Router},
        serde_yaml,
        utoipa::openapi::OpenApi,
        UnwrapInfallible as _,
    },
    rest::{
        utils::{errors, ApiResult, Path},
        ApiState,
        HasCustomRestApi,
    },
    ApiStateAccessor,
    Spec,
    StateAccessor,
    StateReader,
};
use sov_state::User;

use crate::{criteria::CriteriaProposal, Campaign, Core, Indexer, Power, Relayer, Segment};

// Campaign queries.
impl<S: Spec> Core<S> {
    pub fn get_campaign<Accessor: StateAccessor>(
        &self,
        campaign_id: u64,
        state: &mut Accessor,
    ) -> Result<Option<Campaign<S>>, <Accessor as StateReader<User>>::Error> {
        self.campaigns.get(&campaign_id, state)
    }

    pub fn get_criteria_proposal<Accessor: StateAccessor>(
        &self,
        campaign_id: u64,
        proposal_id: u64,
        state: &mut Accessor,
    ) -> Result<Option<CriteriaProposal<S>>, <Accessor as StateReader<User>>::Error> {
        let proposals = self.criteria_proposals.get(&campaign_id, state)?;
        if proposals.is_none() {
            return Ok(None);
        }

        Ok(proposals.unwrap().get((proposal_id) as usize).cloned())
    }

    pub fn get_segment<Accessor: StateAccessor>(
        &self,
        campaign_id: u64,
        state: &mut Accessor,
    ) -> Result<Option<Segment>, <Accessor as StateReader<User>>::Error> {
        self.segments.get(&campaign_id, state)
    }
}

// Indexer queries.
impl<S: Spec> Core<S> {
    pub fn get_indexer<Accessor: StateAccessor>(
        &self,
        addr: S::Address,
        state: &mut Accessor,
    ) -> Result<Option<Indexer<S>>, <Accessor as StateReader<User>>::Error> {
        Ok(self
            .indexer_aliases
            .get(&addr, state)?
            .map(|alias| Indexer { addr, alias }))
    }

    pub fn get_indexers<Accessor: StateAccessor>(
        &self,
        state: &mut Accessor,
    ) -> Result<Vec<Indexer<S>>, <Accessor as StateReader<User>>::Error> {
        let mut indexers = vec![];

        for addr in self
            .indexers
            .iter(state)?
            .collect::<Result<Vec<_>, <Accessor as StateReader<User>>::Error>>()?
        {
            indexers.push(Indexer {
                addr: addr.clone(),
                alias: self.indexer_aliases.get(&addr, state)?.unwrap_or_default(),
            });
        }

        Ok(indexers)
    }
}

// Relayer queries.
impl<S: Spec> Core<S> {
    pub fn get_relayer<Accessor: StateAccessor>(
        &self,
        addr: S::Address,
        state: &mut Accessor,
    ) -> Result<Option<Relayer<S>>, <Accessor as StateReader<User>>::Error> {
        Ok(self
            .relayers
            .iter(state)?
            .collect::<Result<Vec<_>, <Accessor as StateReader<User>>::Error>>()?
            .into_iter()
            .find(|relayer| addr == *relayer))
    }
}

// Voting queries.
impl<S: Spec> Core<S> {
    pub fn get_voting_power<Accessor: StateAccessor>(
        &self,
        addr: S::Address,
        state: &mut Accessor,
    ) -> Result<Power, <Accessor as StateReader<User>>::Error> {
        Ok(self.powers.get(&addr, state)?.unwrap_or_default())
    }

    pub fn get_voting_powers<Accessor: StateAccessor>(
        &self,
        state: &mut Accessor,
    ) -> Result<Vec<(S::Address, Power)>, <Accessor as StateReader<User>>::Error> {
        Ok(self
            .powers_index
            .iter(state)?
            .collect::<Result<Vec<_>, <Accessor as StateReader<User>>::Error>>()?)
    }
}

// RPC
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

    fn custom_openapi_spec(&self) -> Option<OpenApi> {
        let open_api =
            serde_yaml::from_str(include_str!("../openapi-v3.yaml")).expect("Invalid OpenAPI spec");
        Some(open_api)
    }
}
