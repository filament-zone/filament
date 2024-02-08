#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

use sov_modules_api::{
    CallResponse, Context, Error, Module, ModuleInfo, StateMap, StateValue, WorkingSet,
};

mod call;
mod genesis;
#[cfg(feature = "native")]
mod query;

pub use call::CallMessage;
pub use genesis::*;
#[cfg(feature = "native")]
pub use query::*;

/// Module to track outposts on foreign state machines.
#[cfg_attr(feature = "native", derive(sov_modules_api::ModuleCallJsonSchema))]
#[derive(Clone, ModuleInfo)]
pub struct OutpostRegistry<C>
where
    C: Context,
{
    /// The address of the OutpostsRegistry module.
    #[address]
    address: C::Address,

    /// Admin of the OutpostsRegistry module.
    #[state]
    admin: StateValue<C::Address>,

    #[state]
    outposts: StateMap<String, C::Address>,
}

impl<C> Module for OutpostRegistry<C>
where
    C: Context,
{
    type Context = C;
    type Config = OutpostRegistryConfig<C>;
    type CallMessage = CallMessage;

    type Event = ();

    fn genesis(&self, config: &Self::Config, working_set: &mut WorkingSet<C>) -> Result<(), Error> {
        Ok(self.init_module(config, working_set)?)
    }

    fn call(
        &self,
        msg: Self::CallMessage,
        context: &Self::Context,
        working_set: &mut WorkingSet<C>,
    ) -> Result<CallResponse, Error> {
        let res = match msg {
            CallMessage::Register { chain_id } => self.register(context, working_set, chain_id),
        };
        Ok(res?)
    }
}
