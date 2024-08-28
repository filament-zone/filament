use sov_modules_api::{
    CallResponse,
    Context,
    Error,
    EventEmitter as _,
    Module,
    ModuleId,
    ModuleInfo,
    Spec,
    StateAccessor,
    StateMap,
    StateValue,
    StateVec,
    TxState,
    WorkingSet,
};

mod call;
pub use call::CallMessage;

pub mod campaign;
use campaign::{Campaign, Payment, Phase};

pub mod criteria;
use criteria::{Criteria, CriteriaProposal};

pub mod crypto;

pub mod delegate;
use delegate::Eviction;

mod error;
pub use error::CoreError;

mod event;
pub use event::Event;

mod genesis;
pub use genesis::CoreConfig;

mod indexer;
pub use indexer::{Alias, Indexer};

pub mod playbook;
pub use playbook::{Budget, Playbook};

pub mod segment;
pub use segment::Segment;

#[cfg(feature = "native")]
mod rpc;
#[cfg(feature = "native")]
pub use rpc::*;

#[derive(ModuleInfo)]
pub struct Core<S: Spec> {
    #[id]
    pub(crate) id: ModuleId,

    #[state]
    pub(crate) admin: StateValue<S::Address>,

    #[state]
    pub(crate) delegates: StateVec<S::Address>,

    #[state]
    pub(crate) next_campaign_id: StateValue<u64>,

    #[state]
    pub(crate) campaigns: StateMap<u64, Campaign<S>>,

    #[state]
    pub(crate) criteria_proposals: StateMap<u64, Vec<CriteriaProposal<S>>>,

    #[state]
    pub(crate) indexers: StateVec<S::Address>,

    #[state]
    pub(crate) indexer_aliases: StateMap<S::Address, String>,

    #[state]
    pub(crate) segments: StateMap<u64, Segment>,
}

impl<S: Spec> Module for Core<S> {
    type CallMessage = call::CallMessage<S>;
    type Config = CoreConfig<S>;
    type Event = Event<S>;
    type Spec = S;

    fn genesis(&self, config: &Self::Config, working_set: &mut WorkingSet<S>) -> Result<(), Error> {
        Ok(self.init_module(config, working_set)?)
    }

    fn call(
        &self,
        msg: Self::CallMessage,
        context: &Context<Self::Spec>,
        working_set: &mut impl TxState<S>,
    ) -> Result<CallResponse, Error> {
        match msg {
            call::CallMessage::Init {
                criteria,
                budget,
                payment,
                evictions,
            } => self
                .call_init_campaign(criteria, budget, payment, evictions, context, working_set)
                .map_err(|e| Error::ModuleError(e.into())),
            call::CallMessage::ProposeCriteria {
                campaign_id,
                criteria,
            } => self
                .call_propose_criteria(campaign_id, criteria, context, working_set)
                .map_err(|e| Error::ModuleError(e.into())),
            call::CallMessage::ConfirmCriteria {
                campaign_id,
                proposal_id,
            } => self
                .call_confirm_criteria(campaign_id, proposal_id, context, working_set)
                .map_err(|e| Error::ModuleError(e.into())),
            call::CallMessage::RejectCriteria { id } => self
                .call_reject_criteria(id, context, working_set)
                .map_err(|e| Error::ModuleError(e.into())),

            call::CallMessage::IndexCampaign { id } => self
                .call_index_campaign(id, context, working_set)
                .map_err(|e| Error::ModuleError(e.into())),
            call::CallMessage::PostSegment { id, segment } => self
                .call_post_segment(id, segment, context, working_set)
                .map_err(|e| Error::ModuleError(e.into())),

            call::CallMessage::RegisterIndexer(addr, alias) => self
                .call_register_indexer(addr, alias, context, working_set)
                .map_err(|e| Error::ModuleError(e.into())),
            call::CallMessage::UnregisterIndexer(addr) => self
                .call_unregister_indexer(addr, context, working_set)
                .map_err(|e| Error::ModuleError(e.into())),
        }
    }
}

// Campaign handlers.
impl<S: Spec> Core<S> {
    fn init_campaign(
        &self,
        sender: S::Address,
        criteria: Criteria,
        budget: Budget,
        payment: Option<Payment>,
        evictions: Vec<Eviction<S>>,
        working_set: &mut impl TxState<S>,
    ) -> Result<(), CoreError<S>> {
        tracing::info!(%sender, "Init campaign request");

        // TODO(xla): Expect bond and assert bond is locked for sender.
        // TODO(xla): Only accept if sender is a valid campaigner.
        // TODO(xla): Check that all Coins exist and amounts are payable.

        let id = self
            .next_campaign_id
            .get(working_set)
            .ok_or(CoreError::NextIdMissing)?;

        if self.campaigns.get(&id, working_set).is_some() {
            return Err(CoreError::IdExists { id });
        }

        if criteria.len() == 0 {
            return Err(CoreError::MissingCriteria);
        }

        // TODO(xla): Compute list of proposed delegates based on matching criteria.
        let proposed_delegates = self.delegates.iter(working_set).collect::<Vec<_>>();

        if !evictions.iter().all(|e| proposed_delegates.contains(e)) {
            return Err(CoreError::InvalidEviction);
        }

        let delegates = {
            let mut delegates = proposed_delegates.clone();
            delegates.retain(|d| !evictions.contains(d));
            delegates
        };

        // TODO(xla): Settle payment in case of evictions.
        let mut payments = vec![];
        if let Some(ref payment) = payment {
            payments.push(payment.clone());
        }

        self.campaigns.set(
            &id,
            &Campaign {
                campaigner: sender.clone(),
                phase: Phase::Criteria,

                criteria,
                budget,
                payments,

                proposed_delegates,
                evictions: evictions.clone(),
                delegates,

                indexer: None,
            },
            working_set,
        );
        self.next_campaign_id.set(&(id + 1), working_set);

        self.emit_event(
            working_set,
            "campaign_initialized",
            Event::CampaignInitialized {
                id,
                campaigner: sender,
                payment,
                evictions,
            },
        );

        tracing::info!(%id, "Campaign initialized");

        Ok(())
    }

    fn propose_criteria(
        &self,
        sender: S::Address,
        campaign_id: u64,
        criteria: Criteria,
        working_set: &mut impl TxState<S>,
    ) -> Result<(), CoreError<S>> {
        tracing::info!(%sender, %campaign_id, "Criteria propose request");

        let campaign = self
            .campaigns
            .get(&campaign_id, working_set)
            .ok_or(CoreError::CampaignNotFound { id: campaign_id })?;

        if campaign.phase != Phase::Criteria {
            return Err(CoreError::InvalidCriteriaProposal { campaign_id });
        }

        if !campaign.delegates.contains(&sender) {
            return Err(CoreError::InvalidProposer {
                reason: format!("{} is not a campaign delegate", sender),
            });
        }

        let mut proposals = self
            .criteria_proposals
            .get(&campaign_id, working_set)
            .unwrap_or_default();
        let proposal_id = (proposals.len()) as u64;

        proposals.push(CriteriaProposal {
            campaign_id: proposal_id,
            proposer: sender.clone(),
            criteria,
        });

        self.criteria_proposals
            .set(&campaign_id, &proposals, working_set);

        self.emit_event(
            working_set,
            "criteria_proposed",
            Event::CriteriaProposed {
                campaign_id,
                proposer: sender,
                proposal_id,
            },
        );

        tracing::info!(%campaign_id, %proposal_id, "Criteria proposed");

        Ok(())
    }

    fn confirm_criteria(
        &self,
        sender: S::Address,
        campaign_id: u64,
        proposal_id: Option<u64>,
        working_set: &mut impl TxState<S>,
    ) -> Result<(), CoreError<S>> {
        tracing::info!(%sender, %campaign_id, "Criteria confirm request");

        let mut campaign = self
            .campaigns
            .get(&campaign_id, working_set)
            .ok_or(CoreError::CampaignNotFound { id: campaign_id })?;

        if campaign.campaigner != sender {
            return Err(CoreError::SenderNotCampaigner { sender });
        }

        if campaign.phase != Phase::Criteria {
            return Err(CoreError::InvalidCriteriaProposal { campaign_id });
        }

        campaign.phase = Phase::Publish;

        self.campaigns.set(&campaign_id, &campaign, working_set);

        self.emit_event(
            working_set,
            "criteria_confirmed",
            Event::CriteriaConfirmed {
                campaign_id,
                proposal_id,
            },
        );

        tracing::info!(%campaign_id, ?proposal_id, "Criteria proposed");

        Ok(())
    }

    fn reject_criteria(
        &self,
        _sender: S::Address,
        _id: u64,
        _working_set: &mut impl TxState<S>,
    ) -> Result<(), CoreError<S>> {
        todo!()
    }

    fn index_campaign(
        &self,
        sender: S::Address,
        id: u64,
        working_set: &mut impl TxState<S>,
    ) -> Result<(), CoreError<S>> {
        tracing::info!(%sender, %id, "Index campaign request");

        let mut campaign = self
            .campaigns
            .get(&id, working_set)
            .ok_or(CoreError::CampaignNotFound { id })?;

        if campaign.indexer.is_none() || sender != campaign.indexer.clone().unwrap() {
            return Err(CoreError::IndexerMissmatch {
                id,
                indexer: campaign.indexer,
                sender,
            });
        }
        if campaign.phase != Phase::Publish {
            return Err(CoreError::InvalidTransition {
                id,
                current: campaign.phase,
                attempted: Phase::Indexing,
            });
        }

        campaign.phase = Phase::Indexing;
        self.campaigns.set(&id, &campaign, working_set);

        self.emit_event(
            working_set,
            "campaign_indexing",
            Event::CampaignIndexing {
                id,
                indexer: sender,
            },
        );

        tracing::info!(%id, "Campaign indexing");

        Ok(())
    }

    fn post_segment(
        &self,
        sender: S::Address,
        id: u64,
        segment: Segment,
        working_set: &mut impl TxState<S>,
    ) -> Result<(), CoreError<S>> {
        tracing::info!(%sender, %id, "Post segment request");

        let mut campaign = self
            .campaigns
            .get(&id, working_set)
            .ok_or(CoreError::CampaignNotFound { id })?;

        if campaign.indexer.is_none() || sender != campaign.indexer.clone().unwrap() {
            return Err(CoreError::IndexerMissmatch {
                id,
                indexer: campaign.indexer,
                sender,
            });
        }
        if campaign.phase != Phase::Indexing {
            return Err(CoreError::InvalidTransition {
                id,
                current: campaign.phase,
                attempted: Phase::Distribution,
            });
        }
        if self.segments.get(&id, working_set).is_some() {
            return Err(CoreError::SegmentExists { id });
        }

        campaign.phase = Phase::Distribution;
        self.campaigns.set(&id, &campaign, working_set);
        self.segments.set(&id, &segment, working_set);

        self.emit_event(
            working_set,
            "segment_posted",
            Event::SegmentPosted {
                id,
                indexer: sender.clone(),
            },
        );

        tracing::info!(%sender, %id, "Segment posted");

        Ok(())
    }
}

// Indexer handlers.
impl<S: Spec> Core<S> {
    fn register_indexer(
        &self,
        sender: S::Address,
        indexer: S::Address,
        alias: String,
        working_set: &mut impl TxState<S>,
    ) -> Result<(), CoreError<S>> {
        tracing::info!(%indexer, ?alias, "Register indexer request");

        // Only allow admin to update registry for now.
        let admin = self.admin.get(working_set).ok_or(CoreError::AdminNotSet)?;
        if sender != admin {
            return Err(CoreError::SenderNotAdmin { sender });
        }

        if !self.indexers.iter(working_set).any(|each| each == indexer) {
            self.indexers.push(&indexer, working_set);
        }

        self.indexer_aliases.set(&indexer, &alias, working_set);

        self.emit_event(
            working_set,
            "indexer_registered",
            Event::<S>::IndexerRegistered {
                addr: indexer.clone(),
                alias: alias.clone(),
            },
        );
        tracing::info!(%indexer, ?alias, "Indexer registered");

        Ok(())
    }

    fn unregister_indexer(
        &self,
        sender: S::Address,
        indexer: S::Address,
        working_set: &mut impl TxState<S>,
    ) -> Result<(), CoreError<S>> {
        tracing::info!(%indexer, "Unregister indexer request");

        let admin = self.admin.get(working_set).ok_or(CoreError::AdminNotSet)?;
        if sender != admin {
            return Err(CoreError::SenderNotAdmin { sender });
        }

        let mut indexers = self.indexers.iter(working_set).collect::<Vec<_>>();
        let pos = indexers.iter().position(|each| *each == indexer).ok_or(
            CoreError::IndexerNotRegistered {
                indexer: indexer.clone(),
            },
        )?;
        indexers.remove(pos);

        self.indexers.set_all(indexers, working_set);

        self.emit_event(
            working_set,
            "indexer_unregistered",
            Event::IndexerUnregistered {
                addr: indexer.clone(),
            },
        );
        tracing::info!(%indexer, "Indexer unregistered");

        Ok(())
    }
}

// Queries.
impl<S: Spec> Core<S> {
    pub fn get_campaign(&self, id: u64, working_set: &mut WorkingSet<S>) -> Option<Campaign<S>> {
        self.campaigns.get(&id, working_set)
    }

    pub fn get_criteria_proposal(
        &self,
        campaign_id: u64,
        proposal_id: u64,
        working_set: &mut WorkingSet<S>,
    ) -> Option<CriteriaProposal<S>> {
        let proposals = self.criteria_proposals.get(&campaign_id, working_set);
        if proposals.is_none() {
            return None;
        }

        proposals.unwrap().get((proposal_id) as usize).cloned()
    }

    pub fn get_indexer(
        &self,
        addr: S::Address,
        working_set: &mut impl StateAccessor,
    ) -> Option<Indexer<S>> {
        let alias = self.indexer_aliases.get(&addr, working_set)?;
        Some(Indexer { addr, alias })
    }

    pub fn get_indexers(&self, working_set: &mut WorkingSet<S>) -> Vec<Indexer<S>> {
        let addrs = self.indexers.iter(working_set).collect::<Vec<_>>();
        addrs
            .iter()
            .map(|addr| Indexer {
                addr: addr.clone(),
                alias: self
                    .indexer_aliases
                    .get(addr, working_set)
                    .unwrap_or_default(),
            })
            .collect::<Vec<_>>()
    }

    pub fn get_segment(&self, id: u64, working_set: &mut WorkingSet<S>) -> Option<Segment> {
        self.segments.get(&id, working_set)
    }
}
