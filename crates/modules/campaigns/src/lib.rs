use filament_hub_indexer_registry::IndexerRegistry;
use segment::Segment;
use serde::{Deserialize, Serialize};
use sov_modules_api::{
    CallResponse,
    Context,
    Error,
    EventEmitter as _,
    Module,
    ModuleId,
    ModuleInfo,
    Spec,
    StateMap,
    StateValue,
    TxState,
    WorkingSet,
};

mod call;
pub use call::CallMessage;

pub mod campaign;
use campaign::{Campaign, ChainId, Status};

pub mod crypto;

mod error;
pub use error::CampaignsError;

mod event;
pub use event::Event;

mod genesis;

pub mod playbook;

pub mod segment;

#[cfg(feature = "native")]
mod rpc;
#[cfg(feature = "native")]
pub use rpc::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignsConfig<S: Spec> {
    pub campaigns: Vec<Campaign<S>>,
}

#[derive(ModuleInfo)]
pub struct Campaigns<S: Spec> {
    #[id]
    pub(crate) id: ModuleId,

    #[module]
    pub indexer_registry: IndexerRegistry<S>,

    #[state]
    pub(crate) next_id: StateValue<u64>,

    #[state]
    pub(crate) campaigns: StateMap<u64, Campaign<S>>,

    #[state]
    pub(crate) campaigns_by_origin: StateMap<(ChainId, u64), u64>,

    #[state]
    pub(crate) segments: StateMap<u64, Segment>,
}

impl<S: Spec> Module for Campaigns<S> {
    type CallMessage = call::CallMessage<S>;
    type Config = CampaignsConfig<S>;
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
            call::CallMessage::CreateCampaign {
                origin,
                origin_id,

                indexer,
                attester,

                playbook,
            } => self
                .call_create_campaign(
                    origin,
                    origin_id,
                    indexer,
                    attester,
                    playbook,
                    context,
                    working_set,
                )
                .map_err(|e| Error::ModuleError(e.into())),
            call::CallMessage::IndexCampaign { id } => self
                .call_index_campaign(id, context, working_set)
                .map_err(|e| Error::ModuleError(e.into())),
            call::CallMessage::PostSegment { id, segment } => self
                .call_post_segment(id, segment, context, working_set)
                .map_err(|e| Error::ModuleError(e.into())),
        }
    }
}

impl<S: Spec> Campaigns<S> {
    #[allow(clippy::too_many_arguments)]
    fn create_campaign(
        &self,
        sender: S::Address,
        origin_id: u64,
        origin: ChainId,
        indexer: S::Address,
        attester: S::Address,
        playbook: playbook::Playbook,
        working_set: &mut impl TxState<S>,
    ) -> Result<(), CampaignsError<S>> {
        tracing::info!(%sender, %origin, %origin_id, "Create campaign request");

        // TODO(xla): Only accept if sender is a trusted oracle.
        // TODO(xla): Validate that attester is registered.
        // TODO(xla): Check that all Coins exist and amount are payable.

        self.indexer_registry
            .get_indexer(indexer.clone(), working_set)
            .ok_or(CampaignsError::<S>::IndexerNotRegistered {
                indexer: indexer.clone(),
            })?;

        if self
            .campaigns_by_origin
            .get(&(origin.clone(), origin_id), working_set)
            .is_some()
        {
            return Err(CampaignsError::CampaignExists { origin, origin_id });
        }

        let id = self
            .next_id
            .get(working_set)
            .ok_or(CampaignsError::NextIdMissing)?;

        if self.campaigns.get(&id, working_set).is_some() {
            return Err(CampaignsError::IdExists { id });
        }

        self.campaigns.set(
            &id,
            &Campaign {
                status: campaign::Status::Funded,
                origin: origin.clone(),
                origin_id,
                indexer,
                attester,
                playbook,
            },
            working_set,
        );
        self.campaigns_by_origin
            .set(&(origin.clone(), origin_id), &id, working_set);
        self.next_id.set(&(id + 1), working_set);

        self.emit_event(
            working_set,
            "campaign_created",
            Event::CampaignCreated {
                id,
                origin: origin.clone(),
                origin_id,
            },
        );

        tracing::info!(%id, %origin, %origin_id, "Campaign created");

        Ok(())
    }

    fn index_campaign(
        &self,
        sender: S::Address,
        id: u64,
        working_set: &mut impl TxState<S>,
    ) -> Result<(), CampaignsError<S>> {
        tracing::info!(%sender, %id, "Index campaign request");

        let mut campaign = self
            .campaigns
            .get(&id, working_set)
            .ok_or(CampaignsError::CampaignNotFound { id })?;

        if sender != campaign.indexer {
            return Err(CampaignsError::IndexerMissmatch {
                id,
                indexer: campaign.indexer,
                sender,
            });
        }
        if campaign.status != Status::Funded {
            return Err(CampaignsError::InvalidTransition {
                id,
                current: campaign.status,
                attempted: Status::Indexing,
            });
        }

        campaign.status = Status::Indexing;
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
    ) -> Result<(), CampaignsError<S>> {
        tracing::info!(%sender, %id, "Post segment request");

        let mut campaign = self
            .campaigns
            .get(&id, working_set)
            .ok_or(CampaignsError::CampaignNotFound { id })?;

        if sender != campaign.indexer {
            return Err(CampaignsError::IndexerMissmatch {
                id,
                indexer: campaign.indexer,
                sender,
            });
        }
        if campaign.status != Status::Indexing {
            return Err(CampaignsError::InvalidTransition {
                id,
                current: campaign.status,
                attempted: Status::Attesting,
            });
        }
        if self.segments.get(&id, working_set).is_some() {
            return Err(CampaignsError::SegmentExists { id });
        }

        campaign.status = Status::Attesting;
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

impl<S: Spec> Campaigns<S> {
    pub fn get_campaign(&self, id: u64, working_set: &mut WorkingSet<S>) -> Option<Campaign<S>> {
        self.campaigns.get(&id, working_set)
    }

    pub fn get_segment(&self, id: u64, working_set: &mut WorkingSet<S>) -> Option<Segment> {
        self.segments.get(&id, working_set)
    }
}
