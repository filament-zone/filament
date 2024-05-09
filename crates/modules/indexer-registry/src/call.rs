use std::fmt::Debug;

use sov_modules_api::{CallResponse, Context, Spec, TxState};

use crate::{error::IndexerRegistryError, IndexerRegistry};

/// This enumeration represents the available call messages for interacting with
/// the `IndexerRegistry` module.
/// The `derive` for [`schemars::JsonSchema`] is a requirement of
/// [`sov_modules_api::ModuleCallJsonSchema`].
#[cfg_attr(
    feature = "native",
    derive(sov_modules_api::macros::CliWalletArg),
    derive(schemars::JsonSchema),
    schemars(bound = "S::Address: ::schemars::JsonSchema", rename = "CallMessage")
)]
#[derive(
    Clone,
    Debug,
    PartialEq,
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Deserialize,
    serde::Serialize,
)]
pub enum CallMessage<S: Spec> {
    RegisterIndexer(S::Address, String),
    UnregisterIndexer(S::Address),
}

impl<S: Spec> IndexerRegistry<S> {
    pub(crate) fn register(
        &self,
        indexer: S::Address,
        alias: String,
        context: &Context<S>,
        working_set: &mut impl TxState<S>,
    ) -> Result<CallResponse, IndexerRegistryError<S>> {
        self.register_indexer(context.sender().clone(), indexer, alias, working_set)?;
        Ok(CallResponse::default())
    }

    pub(crate) fn unregister(
        &self,
        indexer: S::Address,
        context: &Context<S>,
        working_set: &mut impl TxState<S>,
    ) -> Result<CallResponse, IndexerRegistryError<S>> {
        self.unregister_indexer(context.sender().clone(), indexer, working_set)?;
        Ok(CallResponse::default())
    }
}
