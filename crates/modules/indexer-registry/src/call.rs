use std::fmt::Debug;

use anyhow::{bail, Result};
use sov_modules_api::{CallResponse, Context, EventEmitter, Spec, TxState};

use crate::{event::Event, IndexerRegistry};

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
#[derive(borsh::BorshDeserialize, borsh::BorshSerialize, Debug, PartialEq)]
pub enum CallMessage<S: Spec> {
    RegisterIndexer(S::Address, String),
    UnregisterIndexer(S::Address),
}

impl<S: Spec> IndexerRegistry<S> {
    pub(crate) fn register_indexer(
        &self,
        context: &Context<S>,
        working_set: &mut impl TxState<S>,
        addr: S::Address,
        alias: String,
    ) -> Result<CallResponse> {
        tracing::info!(%addr, ?alias, "Register indexer request");

        // Only allow admin to update registry for now.
        let admin = self.admin.get(working_set);
        if admin.is_none() {
            bail!("Admin needs to be set");
        }
        if *context.sender() != admin.unwrap() {
            bail!("Only admin is allowed to register indexers");
        }

        if !self.indexers.iter(working_set).any(|each| each == addr) {
            self.indexers.push(&addr, working_set);
        }

        self.aliases.set(&addr, &alias, working_set);

        self.emit_event(
            working_set,
            "indexer_registered",
            Event::IndexerRegistered {
                addr: addr.to_string(),
                alias: alias.clone(),
            },
        );
        tracing::info!(%addr, ?alias, "Indexer registered");

        Ok(CallResponse::default())
    }

    pub(crate) fn unregister_indexer(
        &self,
        context: &Context<S>,
        working_set: &mut impl TxState<S>,
        addr: S::Address,
    ) -> Result<CallResponse> {
        tracing::info!(%addr, "Unregister indexer request");

        // Only allow admin to update registry for now.
        let admin = self.admin.get(working_set);
        if admin.is_none() {
            bail!("Admin needs to be set");
        }
        if *context.sender() != admin.unwrap() {
            bail!("Only admin is allowed to register indexers");
        }

        let mut indexers = self.indexers.iter(working_set).collect::<Vec<_>>();
        let pos = indexers.iter().position(|each| *each == addr);
        if pos.is_none() {
            bail!("Indexer is not registered");
        }
        indexers.remove(pos.unwrap());

        self.indexers.set_all(indexers, working_set);

        self.emit_event(
            working_set,
            "indexer_unregistered",
            Event::IndexerUnregistered {
                addr: addr.to_string(),
            },
        );
        tracing::info!(%addr, "Indexer unregistered");

        Ok(CallResponse::default())
    }
}
