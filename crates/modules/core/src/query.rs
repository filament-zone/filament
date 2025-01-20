use std::collections::HashMap;

use jsonrpsee::core::RpcResult;
use sov_modules_api::{
    macros::rpc_gen,
    prelude::{
        axum::{http::Method, routing::get, Router},
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
    CryptoSpec,
    Spec,
    StateAccessor,
    StateReader,
};
use sov_state::User;
use tower_http::cors::{Any, CorsLayer};

use crate::{
    account::Account,
    campaign::Phase,
    criteria::{Criteria, CriteriaProposal},
    voting::{CriteriaVote, DistributionVote},
    Campaign,
    Core,
    Indexer,
    Power,
    Relayer,
    Segment,
};

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize, ts_rs::TS)]
#[ts(export_to = "../../../../bindings/CampaignResponse.ts")]
pub struct CampaignResponse {
    pub id: u64,

    pub campaigner: String,
    pub phase: Phase,

    pub title: String,
    pub description: String,

    pub criteria: Criteria,

    pub evictions: Vec<String>,
    pub delegates: HashMap<String, u64>,

    pub indexer: Option<String>,
}

// Account queries.
impl<S: Spec> Core<S> {
    pub fn get_account_by_eth_addr<Accessor: StateAccessor>(
        &self,
        eth_addr: &str,
        state: &mut Accessor,
    ) -> Result<Option<Account>, <Accessor as StateReader<User>>::Error> {
        let addr = filament_hub_eth::addr_to_hub_address::<S>(eth_addr).unwrap();
        let credential_id = filament_hub_eth::hub_addr_to_credential_id::<
            <S::CryptoSpec as CryptoSpec>::Hasher,
            S,
        >(&addr);
        let account = self
            .nonces
            .nonce(&credential_id, state)?
            .map(|nonce| Account { nonce });
        Ok(account)
    }
}

// Campaign queries.
impl<S: Spec> Core<S> {
    pub fn get_campaign<Accessor: StateAccessor>(
        &self,
        campaign_id: u64,
        state: &mut Accessor,
    ) -> Result<Option<Campaign<S>>, <Accessor as StateReader<User>>::Error> {
        self.campaigns.get(&campaign_id, state)
    }

    pub fn get_campaigns<Accessor: StateAccessor>(
        &self,
        state: &mut Accessor,
    ) -> Result<Vec<Campaign<S>>, <Accessor as StateReader<User>>::Error> {
        let ids = self
            .campaigns_index
            .iter(state)?
            .collect::<Result<Vec<_>, _>>()?;
        let mut campaigns = vec![];

        for id in &ids {
            if let Some(campaign) = self.campaigns.get(id, state)? {
                campaigns.push(campaign);
            }
        }

        Ok(campaigns)
    }

    pub fn get_campaigns_by_addr<Accessor: StateAccessor>(
        &self,
        addr: S::Address,
        state: &mut Accessor,
    ) -> Result<Vec<Campaign<S>>, <Accessor as StateReader<User>>::Error> {
        let ids = self
            .campaigns_by_addr
            .get(&addr, state)?
            .unwrap_or_default();
        let mut campaigns = vec![];
        for id in &ids {
            if let Some(campaign) = self.campaigns.get(id, state)? {
                campaigns.push(campaign);
            }
        }

        Ok(campaigns)
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

    pub fn get_criteria_votes<Accessor: StateAccessor>(
        &self,
        campaign_id: u64,
        state: &mut Accessor,
    ) -> Result<HashMap<String, CriteriaVote>, <Accessor as StateReader<User>>::Error> {
        Ok(self
            .criteria_votes
            .get(&campaign_id, state)?
            .unwrap_or_default())
    }

    pub fn get_distribution_votes<Accessor: StateAccessor>(
        &self,
        campaign_id: u64,
        state: &mut Accessor,
    ) -> Result<HashMap<String, DistributionVote>, <Accessor as StateReader<User>>::Error> {
        Ok(self
            .distribution_votes
            .get(&campaign_id, state)?
            .unwrap_or_default())
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
#[allow(clippy::type_complexity)]
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
        self.powers_index
            .iter(state)?
            .collect::<Result<Vec<_>, <Accessor as StateReader<User>>::Error>>()
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
    fn campaign_to_response<Accessor: StateAccessor>(
        &self,
        campaign: Campaign<S>,
        state: &mut Accessor,
    ) -> anyhow::Result<CampaignResponse> {
        let mut evictions = vec![];
        for addr in &campaign.evictions {
            evictions.push(self.eth_addresses.get(addr, state)?.unwrap());
        }
        let mut delegates = HashMap::new();
        for (addr, power) in &campaign.delegates {
            delegates.insert(
                self.eth_addresses
                    .get(&(*addr).parse::<S::Address>()?, state)?
                    .unwrap(),
                *power,
            );
        }
        let mut indexer = None;
        if campaign.indexer.is_some() {
            indexer = Some(
                self.eth_addresses
                    .get(&campaign.indexer.unwrap(), state)?
                    .unwrap(),
            );
        }

        Ok(CampaignResponse {
            id: campaign.id,
            campaigner: self
                .eth_addresses
                .get(&campaign.campaigner, state)?
                .unwrap(),
            phase: campaign.phase,
            title: campaign.title,
            description: campaign.description,
            criteria: campaign.criteria,
            evictions,
            delegates,
            indexer,
        })
    }

    async fn route_get_account_by_eth_addr(
        state: ApiState<Self, S>,
        Path(eth_addr): Path<String>,
    ) -> ApiResult<Account> {
        let account = state
            .get_account_by_eth_addr(&eth_addr, &mut state.api_state_accessor())
            .unwrap_infallible()
            .ok_or_else(|| errors::not_found_404("Account", eth_addr))?;
        Ok(account.into())
    }

    #[allow(clippy::single_call_fn, clippy::unused_async)]
    async fn route_get_campaign(
        state: ApiState<Self, S>,
        Path(campaign_id): Path<u64>,
    ) -> ApiResult<CampaignResponse> {
        let campaign = state
            .get_campaign(campaign_id, &mut state.api_state_accessor())
            .unwrap_infallible()
            .ok_or_else(|| errors::not_found_404("Campaign", campaign_id))?;

        state
            .campaign_to_response(campaign, &mut state.api_state_accessor())
            .map(Into::into)
            .map_err(|err| {
                errors::internal_server_error_response_500(format!(
                    "failed to populate campaign response: {}",
                    err
                ))
            })
    }

    async fn route_get_campaigns(state: ApiState<Self, S>) -> ApiResult<Vec<CampaignResponse>> {
        let mut campaigns = vec![];
        for campaign in state
            .get_campaigns(&mut state.api_state_accessor())
            .unwrap_infallible()
            .into_iter()
        {
            campaigns.push(
                state
                    .campaign_to_response(campaign, &mut state.api_state_accessor())
                    .map_err(|err| {
                        errors::internal_server_error_response_500(format!(
                            "failed to populate campaign response: {}",
                            err
                        ))
                    })?,
            );
        }

        Ok(campaigns.into())
    }

    async fn route_get_campaigns_by_addr(
        state: ApiState<Self, S>,
        Path(addr): Path<S::Address>,
    ) -> ApiResult<Vec<CampaignResponse>> {
        let mut campaigns = vec![];
        for campaign in state
            .get_campaigns_by_addr(addr, &mut state.api_state_accessor())
            .unwrap_infallible()
            .into_iter()
        {
            campaigns.push(
                state
                    .campaign_to_response(campaign, &mut state.api_state_accessor())
                    .map_err(|err| {
                        errors::internal_server_error_response_500(format!(
                            "failed to populate campaign response: {}",
                            err
                        ))
                    })?,
            );
        }

        Ok(campaigns.into())
    }

    async fn route_get_campaigns_by_eth_addr(
        state: ApiState<Self, S>,
        Path(eth_addr): Path<String>,
    ) -> ApiResult<Vec<CampaignResponse>> {
        let addr = filament_hub_eth::addr_to_hub_address::<S>(&eth_addr)
            .map_err(|e| errors::bad_request_400("malformed address", e))?;

        let mut campaigns = vec![];
        for campaign in state
            .get_campaigns_by_addr(addr, &mut state.api_state_accessor())
            .unwrap_infallible()
            .into_iter()
        {
            campaigns.push(
                state
                    .campaign_to_response(campaign, &mut state.api_state_accessor())
                    .map_err(|err| {
                        errors::internal_server_error_response_500(format!(
                            "failed to populate campaign response: {}",
                            err
                        ))
                    })?,
            );
        }

        Ok(campaigns.into())
    }

    async fn route_get_criteria_votes(
        state: ApiState<Self, S>,
        Path(campaign_id): Path<u64>,
    ) -> ApiResult<HashMap<String, CriteriaVote>> {
        Ok(state
            .get_criteria_votes(campaign_id, &mut state.api_state_accessor())
            .unwrap_infallible()
            .into())
    }

    async fn route_get_distribution_votes(
        state: ApiState<Self, S>,
        Path(campaign_id): Path<u64>,
    ) -> ApiResult<HashMap<String, DistributionVote>> {
        Ok(state
            .get_distribution_votes(campaign_id, &mut state.api_state_accessor())
            .unwrap_infallible()
            .into())
    }
}

impl<S: Spec> HasCustomRestApi for Core<S> {
    type Spec = S;

    fn custom_rest_api(&self, state: ApiState<Self, Self::Spec>) -> Router<()> {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(vec![Method::GET, Method::OPTIONS])
            .allow_headers(Any);

        Router::new()
            .route(
                "/accounts/by_eth_addr/:eth_addr",
                get(Self::route_get_account_by_eth_addr),
            )
            .route(
                "/campaigns/by_addr/:addr}",
                get(Self::route_get_campaigns_by_addr),
            )
            .route(
                "/campaigns/by_eth_addr/:eth_addr}",
                get(Self::route_get_campaigns_by_eth_addr),
            )
            .route(
                "/campaigns/:campaignId/criteria/votes",
                get(Self::route_get_criteria_votes),
            )
            .route(
                "/campaigns/:campaignId/distribution/votes",
                get(Self::route_get_distribution_votes),
            )
            .route("/campaigns/:campaignId", get(Self::route_get_campaign))
            .route("/campaigns", get(Self::route_get_campaigns))
            .layer(cors)
            .with_state(state)
    }

    fn custom_openapi_spec(&self) -> Option<OpenApi> {
        let open_api =
            serde_yaml::from_str(include_str!("../openapi-v3.yaml")).expect("Invalid OpenAPI spec");
        Some(open_api)
    }
}
