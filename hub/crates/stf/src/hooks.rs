//! This module implements the various "hooks" that are called by the STF during execution.
//! These hooks can be used to add custom logic at various points in the slot lifecycle:
//! - Before and after each transaction is executed.
//! - At the beginning and end of each batch ("blob")
//! - At the beginning and end of each slot (DA layer block)

use sov_accounts::AccountsTxHook;
use sov_bank::BankTxHook;
use sov_modules_api::{
    hooks::{ApplyBlobHooks, FinalizeHook, SlotHooks, TxHooks},
    transaction::Transaction,
    AccessoryWorkingSet,
    BlobReaderTrait,
    Context,
    DaSpec,
    Spec,
    WorkingSet,
};
use sov_modules_stf_blueprint::{RuntimeTxHook, SequencerOutcome};
use sov_sequencer_registry::SequencerRegistry;
use sov_state::Storage;
use tracing::info;

use super::runtime::Runtime;

impl<C: Context, Da: DaSpec> TxHooks for Runtime<C, Da> {
    type Context = C;
    type PreArg = RuntimeTxHook<C>;
    type PreResult = C;

    fn pre_dispatch_tx_hook(
        &self,
        tx: &Transaction<Self::Context>,
        working_set: &mut WorkingSet<C>,
        arg: &RuntimeTxHook<C>,
    ) -> anyhow::Result<C> {
        let RuntimeTxHook { height, sequencer } = arg;
        let AccountsTxHook { sender, sequencer } =
            self.accounts
                .pre_dispatch_tx_hook(tx, working_set, sequencer)?;

        let hook = BankTxHook { sender, sequencer };
        self.bank.pre_dispatch_tx_hook(tx, working_set, &hook)?;

        Ok(C::new(hook.sender, hook.sequencer, *height))
    }

    fn post_dispatch_tx_hook(
        &self,
        tx: &Transaction<Self::Context>,
        ctx: &C,
        working_set: &mut WorkingSet<C>,
    ) -> anyhow::Result<()> {
        self.accounts.post_dispatch_tx_hook(tx, ctx, working_set)?;
        self.bank.post_dispatch_tx_hook(tx, ctx, working_set)?;
        Ok(())
    }
}

impl<C: Context, Da: DaSpec> ApplyBlobHooks<Da::BlobTransaction> for Runtime<C, Da> {
    type BlobResult =
        SequencerOutcome<<<Da as DaSpec>::BlobTransaction as BlobReaderTrait>::Address>;
    type Context = C;

    fn begin_blob_hook(
        &self,
        blob: &mut Da::BlobTransaction,
        working_set: &mut WorkingSet<C>,
    ) -> anyhow::Result<()> {
        // Before executing each batch, check that the sender is regsitered as a sequencer
        self.sequencer_registry.begin_blob_hook(blob, working_set)
    }

    fn end_blob_hook(
        &self,
        result: Self::BlobResult,
        working_set: &mut WorkingSet<C>,
    ) -> anyhow::Result<()> {
        // After processing each blob, reward or slash the sequencer if appropriate
        match result {
            SequencerOutcome::Rewarded(_reward) => {
                // TODO: Process reward here or above.
                <SequencerRegistry<C, Da> as ApplyBlobHooks<Da::BlobTransaction>>::end_blob_hook(
                    &self.sequencer_registry,
                    sov_sequencer_registry::SequencerOutcome::Completed,
                    working_set,
                )
            },
            SequencerOutcome::Ignored => Ok(()),
            SequencerOutcome::Slashed {
                reason,
                sequencer_da_address,
            } => {
                info!("Sequencer {} slashed: {:?}", sequencer_da_address, reason);
                <SequencerRegistry<C, Da> as ApplyBlobHooks<Da::BlobTransaction>>::end_blob_hook(
                    &self.sequencer_registry,
                    sov_sequencer_registry::SequencerOutcome::Slashed {
                        sequencer: sequencer_da_address,
                    },
                    working_set,
                )
            },
        }
    }
}

impl<C: Context, Da: DaSpec> SlotHooks<Da> for Runtime<C, Da> {
    type Context = C;

    fn begin_slot_hook(
        &self,
        _slot_header: &Da::BlockHeader,
        _validity_condition: &Da::ValidityCondition,
        _pre_state_root: &<<Self::Context as Spec>::Storage as Storage>::Root,
        _working_set: &mut sov_modules_api::WorkingSet<C>,
    ) {
    }

    fn end_slot_hook(&self, _working_set: &mut sov_modules_api::WorkingSet<C>) {}
}

impl<C: Context, Da: sov_modules_api::DaSpec> FinalizeHook<Da> for Runtime<C, Da> {
    type Context = C;

    fn finalize_hook(
        &self,
        _root_hash: &<<Self::Context as Spec>::Storage as Storage>::Root,
        _accessory_working_set: &mut AccessoryWorkingSet<'_, C>,
    ) {
    }
}
