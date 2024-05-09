use serde::{Deserialize, Serialize};
use sov_modules_api::{
    CallResponse,
    Context,
    Error,
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
    type Event = Event;
    type Spec = S;

    fn genesis(&self, config: &Self::Config, working_set: &mut WorkingSet<S>) -> Result<(), Error> {
        // The initialization logic
        Ok(self.init_module(config, working_set)?)
    }

    fn call(
        &self,
        msg: Self::CallMessage,
        context: &Context<Self::Spec>,
        working_set: &mut impl TxState<S>,
    ) -> Result<CallResponse, Error> {
        match msg {
            call::CallMessage::RegisterIndexer(addr, alias) => {
                Ok(self.register_indexer(context, working_set, addr, alias)?)
            },
            call::CallMessage::UnregisterIndexer(addr) => {
                Ok(self.unregister_indexer(context, working_set, addr)?)
            },
        }
    }
}
