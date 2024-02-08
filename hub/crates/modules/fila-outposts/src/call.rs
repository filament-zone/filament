use anyhow::{bail, Result};
#[cfg(feature = "native")]
use sov_modules_api::macros::CliWalletArg;
use sov_modules_api::{CallResponse, Context, StateMapAccessor, WorkingSet};

use crate::OutpostRegistry;

/// Available call messages for interacting with the `fila-outposts` module.
#[cfg_attr(
    feature = "native",
    derive(CliWalletArg),
    derive(schemars::JsonSchema),
    schemars(rename = "CallMessage")
)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize),
    derive(serde::Deserialize)
)]
#[derive(Clone, Debug, PartialEq, borsh::BorshDeserialize, borsh::BorshSerialize)]
pub enum CallMessage {
    /// Register an outpost.
    Register {
        /// Unique identifier for the outpost.
        chain_id: String,
    },
}

impl<C> OutpostRegistry<C>
where
    C: Context,
{
    pub(crate) fn register(
        &self,
        context: &C,
        working_set: &mut WorkingSet<C>,
        chain_id: String,
    ) -> Result<CallResponse> {
        if self.outposts.get(&chain_id, working_set).is_some() {
            bail!("Outpost with chain_id {chain_id} already exists");
        }

        self.outposts.set(&chain_id, context.sender(), working_set);

        working_set.add_event(
            "OutpostRegistry register",
            &format!("An outpost with chain_id {chain_id} was registered"),
        );

        Ok(CallResponse::default())
    }
}
