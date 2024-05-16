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
use campaign::{Campaign, ChainId, Status};

pub mod crypto;

mod error;
pub use error::CoreError;

mod event;
pub use event::Event;

mod genesis;
pub use genesis::CoreConfig;

mod indexer;
pub use indexer::{Alias, Indexer};

pub mod playbook;
pub use playbook::Playbook;

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
    pub(crate) next_campaign_id: StateValue<u64>,

    #[state]
    pub(crate) campaigns: StateMap<u64, Campaign<S>>,

    #[state]
    pub(crate) campaigns_by_origin: StateMap<(ChainId, u64), u64>,

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

            call::CallMessage::RegisterIndexer(addr, alias) => self
                .call_register_indexer(addr, alias, context, working_set)
                .map_err(|e| Error::ModuleError(e.into())),
            call::CallMessage::UnregisterIndexer(addr) => self
                .call_unregister_indexer(addr, context, working_set)
                .map_err(|e| Error::ModuleError(e.into())),
        }
    }
}

impl<S: Spec> Core<S> {
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
    ) -> Result<(), CoreError<S>> {
        tracing::info!(%sender, %origin, %origin_id, "Create campaign request");

        // TODO(xla): Only accept if sender is a trusted oracle.
        // TODO(xla): Validate that attester is registered.
        // TODO(xla): Check that all Coins exist and amount are payable.

        self.indexer_aliases
            .get(&indexer.clone(), working_set)
            .ok_or(CoreError::<S>::IndexerNotRegistered {
                indexer: indexer.clone(),
            })?;

        if self
            .campaigns_by_origin
            .get(&(origin.clone(), origin_id), working_set)
            .is_some()
        {
            return Err(CoreError::CampaignExists { origin, origin_id });
        }

        let id = self
            .next_campaign_id
            .get(working_set)
            .ok_or(CoreError::NextIdMissing)?;

        if self.campaigns.get(&id, working_set).is_some() {
            return Err(CoreError::IdExists { id });
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
        self.next_campaign_id.set(&(id + 1), working_set);

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
    ) -> Result<(), CoreError<S>> {
        tracing::info!(%sender, %id, "Index campaign request");

        let mut campaign = self
            .campaigns
            .get(&id, working_set)
            .ok_or(CoreError::CampaignNotFound { id })?;

        if sender != campaign.indexer {
            return Err(CoreError::IndexerMissmatch {
                id,
                indexer: campaign.indexer,
                sender,
            });
        }
        if campaign.status != Status::Funded {
            return Err(CoreError::InvalidTransition {
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
    ) -> Result<(), CoreError<S>> {
        tracing::info!(%sender, %id, "Post segment request");

        let mut campaign = self
            .campaigns
            .get(&id, working_set)
            .ok_or(CoreError::CampaignNotFound { id })?;

        if sender != campaign.indexer {
            return Err(CoreError::IndexerMissmatch {
                id,
                indexer: campaign.indexer,
                sender,
            });
        }
        if campaign.status != Status::Indexing {
            return Err(CoreError::InvalidTransition {
                id,
                current: campaign.status,
                attempted: Status::Attesting,
            });
        }
        if self.segments.get(&id, working_set).is_some() {
            return Err(CoreError::SegmentExists { id });
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

impl<S: Spec> Core<S> {
    pub fn get_campaign(&self, id: u64, working_set: &mut WorkingSet<S>) -> Option<Campaign<S>> {
        self.campaigns.get(&id, working_set)
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
