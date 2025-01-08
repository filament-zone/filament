#![allow(dead_code)]

use std::collections::HashMap;

use sov_modules_api::{
    macros::ModuleRestApi,
    CallResponse,
    Context,
    Error,
    GenesisState,
    Module,
    ModuleId,
    ModuleInfo,
    Spec,
    StateMap,
    StateValue,
    StateVec,
    TxState,
};

mod account;
pub use account::*;

mod call;
pub use call::*;

pub mod campaign;
use campaign::Campaign;

pub mod criteria;
use criteria::CriteriaProposal;

pub mod crypto;

pub mod delegate;

mod event;
pub use event::Event;

mod genesis;
pub use genesis::CoreConfig;

mod indexer;
pub use indexer::{Alias, Indexer};

pub mod playbook;
pub use playbook::{Budget, Playbook};

#[cfg(feature = "native")]
pub mod query;
#[cfg(feature = "native")]
pub use query::*;

pub mod relayer;
pub use relayer::Relayer;

pub mod segment;
pub use segment::Segment;

pub mod voting;
pub use voting::Power;
use voting::{CriteriaVote, DistributionVote};

#[derive(Clone, ModuleInfo, ModuleRestApi)]
pub struct Core<S: Spec> {
    #[id]
    pub(crate) id: ModuleId,

    #[state]
    pub(crate) admin: StateValue<S::Address>,

    // Campaign
    #[state]
    pub(crate) next_campaign_id: StateValue<u64>,

    #[state]
    pub(crate) campaigns: StateMap<u64, Campaign<S>>,

    #[state]
    pub(crate) campaigns_index: StateVec<u64>,

    #[state]
    pub(crate) campaigns_by_addr: StateMap<S::Address, Vec<u64>>,

    #[state]
    pub(crate) criteria_proposals: StateMap<u64, Vec<CriteriaProposal<S>>>,

    #[state]
    pub(crate) criteria_votes: StateMap<u64, HashMap<String, CriteriaVote>>,

    #[state]
    pub(crate) distribution_votes: StateMap<u64, HashMap<String, DistributionVote>>,

    #[state]
    pub(crate) segments: StateMap<u64, Segment>,

    // Delegate
    #[state]
    pub(crate) delegates: StateVec<S::Address>,

    // Indexer
    #[state]
    pub(crate) indexers: StateVec<S::Address>,

    #[state]
    pub(crate) indexer_aliases: StateMap<S::Address, String>,

    // Relayer
    #[state]
    pub(crate) relayers: StateVec<Relayer<S>>,

    // Voting
    #[state]
    pub(crate) total_voting_power: StateValue<Power>,

    #[state]
    pub(crate) powers: StateMap<S::Address, Power>,

    #[state]
    pub(crate) powers_index: StateVec<(S::Address, Power)>,

    // Accounts
    #[state]
    pub(crate) eth_addresses: StateMap<S::Address, String>,

    #[module]
    pub(crate) nonces: sov_nonces::Nonces<S>,
}

impl<S: Spec> Module for Core<S> {
    type CallMessage = call::CallMessage<S>;
    type Config = CoreConfig<S>;
    type Event = Event<S>;
    type Spec = S;

    fn genesis(
        &self,
        config: &Self::Config,
        state: &mut impl GenesisState<S>,
    ) -> Result<(), Error> {
        Ok(self.init_module(config, state)?)
    }

    fn call(
        &self,
        msg: Self::CallMessage,
        context: &Context<Self::Spec>,
        state: &mut impl TxState<S>,
    ) -> Result<CallResponse, Error> {
        match msg {
            // Campaign
            call::CallMessage::Draft {
                title,
                description,
                criteria,
                evictions,
            } => {
                self.draft_campaign(
                    title,
                    description,
                    criteria,
                    evictions,
                    context.sender(),
                    state,
                )?;
                Ok(CallResponse::default())
            },
            call::CallMessage::Init { campaign_id } => {
                self.init_campaign(campaign_id, context.sender(), state)?;
                Ok(CallResponse::default())
            },
            call::CallMessage::ProposeCriteria {
                campaign_id,
                criteria,
            } => {
                self.propose_criteria(campaign_id, criteria, context.sender(), state)?;
                Ok(CallResponse::default())
            },
            call::CallMessage::VoteCriteria { campaign_id, vote } => {
                self.vote_criteria(campaign_id, vote, context.sender(), state)?;
                Ok(CallResponse::default())
            },
            call::CallMessage::ConfirmCriteria {
                campaign_id,
                proposal_id,
            } => {
                self.confirm_criteria(campaign_id, proposal_id, context.sender(), state)?;
                Ok(CallResponse::default())
            },
            call::CallMessage::RejectCriteria { campaign_id } => {
                self.reject_criteria(campaign_id, context.sender(), state)?;
                Ok(CallResponse::default())
            },
            call::CallMessage::IndexCampaign { campaign_id } => {
                self.index_campaign(campaign_id, context.sender(), state)?;
                Ok(CallResponse::default())
            },
            call::CallMessage::PostSegment {
                campaign_id,
                segment,
            } => {
                self.post_segment(campaign_id, segment, context.sender(), state)?;
                Ok(CallResponse::default())
            },
            call::CallMessage::VoteDistribution { campaign_id, vote } => {
                self.vote_distribution(campaign_id, vote, context.sender(), state)?;
                Ok(CallResponse::default())
            },

            // Indexer
            call::CallMessage::RegisterIndexer { address, alias } => {
                self.register_indexer(address, alias, context.sender().clone(), state)?;
                Ok(CallResponse::default())
            },
            call::CallMessage::UnregisterIndexer { address } => {
                self.unregister_indexer(address, context.sender().clone(), state)?;
                Ok(CallResponse::default())
            },

            // Relayer
            call::CallMessage::RegisterRelayer { address } => {
                self.register_relayer(address, context.sender().clone(), state)?;
                Ok(CallResponse::default())
            },
            call::CallMessage::UnregisterRelayer { address } => {
                self.unregister_relayer(address, context.sender().clone(), state)?;
                Ok(CallResponse::default())
            },

            // Voting
            call::CallMessage::UpdateVotingPower { address, power } => {
                self.update_voting_power(address, power, context.sender().clone(), state)?;
                Ok(CallResponse::default())
            },
        }
    }
}
