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
    StateVec,
    TxState,
    WorkingSet,
};

mod call;
pub use call::CallMessage;

mod error;
pub use error::IndexerRegistryError;

mod event;
pub use event::Event;

mod genesis;

#[cfg(feature = "native")]
mod rpc;
#[cfg(feature = "native")]
pub use rpc::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexerRegistryConfig<S: Spec> {
    pub admin: S::Address,
    pub indexers: Vec<(S::Address, String)>,
}

#[cfg_attr(feature = "native", derive(sov_modules_api::ModuleCallJsonSchema))]
#[derive(ModuleInfo)]
pub struct IndexerRegistry<S: Spec> {
    /// Id of the module.
    #[id]
    pub(crate) id: ModuleId,

    #[state]
    pub(crate) admin: StateValue<S::Address>,

    #[state]
    pub(crate) indexers: StateVec<S::Address>,

    #[state]
    pub(crate) aliases: StateMap<S::Address, String>,
}

impl<S: Spec> Module for IndexerRegistry<S> {
    type CallMessage = call::CallMessage<S>;
    type Config = IndexerRegistryConfig<S>;
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
            call::CallMessage::RegisterIndexer(addr, alias) => self
                .register(addr, alias, context, working_set)
                .map_err(|e| Error::ModuleError(e.into())),
            call::CallMessage::UnregisterIndexer(addr) => self
                .unregister(addr, context, working_set)
                .map_err(|e| Error::ModuleError(e.into())),
        }
    }
}

impl<S: Spec> IndexerRegistry<S> {
    fn register_indexer(
        &self,
        sender: S::Address,
        indexer: S::Address,
        alias: String,
        working_set: &mut impl TxState<S>,
    ) -> Result<(), IndexerRegistryError<S>> {
        tracing::info!(%indexer, ?alias, "register_indexer");

        // Only allow admin to update registry for now.
        let admin = self
            .admin
            .get(working_set)
            .ok_or(IndexerRegistryError::AdminNotSet)?;
        if sender != admin {
            return Err(IndexerRegistryError::SenderNotAdmin { sender });
        }

        if !self.indexers.iter(working_set).any(|each| each == indexer) {
            self.indexers.push(&indexer, working_set);
        }

        self.aliases.set(&indexer, &alias, working_set);

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
    ) -> Result<(), IndexerRegistryError<S>> {
        tracing::info!(%indexer, "Unregister indexer request");

        let admin = self
            .admin
            .get(working_set)
            .ok_or(IndexerRegistryError::AdminNotSet)?;
        if sender != admin {
            return Err(IndexerRegistryError::SenderNotAdmin { sender });
        }

        let mut indexers = self.indexers.iter(working_set).collect::<Vec<_>>();
        let pos = indexers.iter().position(|each| *each == indexer).ok_or(
            IndexerRegistryError::IndexerNotRegistered {
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
