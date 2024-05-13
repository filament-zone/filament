use anyhow::Context as _;
use sov_bank::IntoPayable;
use sov_modules_api::{
    batch::BatchWithId,
    hooks::{ApplyBatchHooks, FinalizeHook, SlotHooks, TxHooks},
    namespaces::Accessory,
    runtime::capabilities::{GasEnforcer, RuntimeAuthorization, SequencerAuthorization},
    transaction::AuthenticatedTransactionData,
    Context,
    Gas,
    ModuleInfo,
    Spec,
    StateCheckpoint,
    StateReaderAndWriter,
    WorkingSet,
};
use sov_modules_stf_blueprint::BatchSequencerOutcome;
use sov_rollup_interface::da::DaSpec;
use sov_sequencer_registry::{SequencerRegistry, SequencerStakeMeter};
use tracing::info;

use crate::runtime::Runtime;

impl<S: Spec, Da: DaSpec> TxHooks for Runtime<S, Da> {
    type Spec = S;
}

impl<S: Spec, Da: DaSpec> ApplyBatchHooks<Da> for Runtime<S, Da> {
    type BatchResult = BatchSequencerOutcome;
    type Spec = S;

    fn begin_batch_hook(
        &self,
        batch: &mut BatchWithId,
        sender: &Da::Address,
        working_set: &mut StateCheckpoint<S>,
    ) -> anyhow::Result<()> {
        // Before executing each batch, check that the sender is registered as a sequencer
        self.sequencer_registry
            .begin_batch_hook(batch, sender, working_set)
    }

    fn end_batch_hook(
        &self,
        result: Self::BatchResult,
        sender: &Da::Address,
        state_checkpoint: &mut StateCheckpoint<S>,
    ) {
        // Since we need to make sure the `StfBlueprint` doesn't depend on the module system, we
        // need to convert the `SequencerOutcome` structures manually.
        match result {
            BatchSequencerOutcome::Rewarded(amount) => {
                info!(%sender, ?amount, "Rewarding sequencer");
                <SequencerRegistry<S, Da> as ApplyBatchHooks<Da>>::end_batch_hook(
                    &self.sequencer_registry,
                    sov_sequencer_registry::SequencerOutcome::Rewarded(amount),
                    sender,
                    state_checkpoint,
                );
            },
            BatchSequencerOutcome::Ignored => {},
            BatchSequencerOutcome::Slashed(reason) => {
                info!(%sender, ?reason, "Slashing sequencer");
                <SequencerRegistry<S, Da> as ApplyBatchHooks<Da>>::end_batch_hook(
                    &self.sequencer_registry,
                    sov_sequencer_registry::SequencerOutcome::Slashed,
                    sender,
                    state_checkpoint,
                );
            },
        }
    }
}

impl<S: Spec, Da: DaSpec> SlotHooks for Runtime<S, Da> {
    type Spec = S;

    fn begin_slot_hook(
        &self,
        #[allow(unused_variables)] pre_state_root: <S as Spec>::VisibleHash,
        #[allow(unused_variables)]
        versioned_working_set: &mut sov_modules_api::VersionedStateReadWriter<
            '_,
            StateCheckpoint<S>,
        >,
    ) {
    }

    fn end_slot_hook(
        &self,
        #[allow(unused_variables)] working_set: &mut sov_modules_api::StateCheckpoint<S>,
    ) {
    }
}

impl<S: Spec, Da: sov_modules_api::DaSpec> FinalizeHook for Runtime<S, Da> {
    type Spec = S;

    fn finalize_hook(
        &self,
        #[allow(unused_variables)] root_hash: S::VisibleHash,
        #[allow(unused_variables)] accessory_state: &mut impl StateReaderAndWriter<Accessory>,
    ) {
    }
}

impl<S: Spec, Da: DaSpec> GasEnforcer<S, Da> for Runtime<S, Da> {
    /// The transaction type that the gas enforcer knows how to parse
    type Tx = AuthenticatedTransactionData<S>;

    /// Reserves enough gas for the transaction to be processed, if possible.
    fn try_reserve_gas(
        &self,
        tx: &Self::Tx,
        context: &Context<S>,
        gas_price: &<S::Gas as Gas>::Price,
        mut state_checkpoint: StateCheckpoint<S>,
    ) -> Result<WorkingSet<S>, StateCheckpoint<S>> {
        match self
            .bank
            .reserve_gas(tx, gas_price, context.sender(), &mut state_checkpoint)
        {
            Ok(gas_meter) => Ok(state_checkpoint.to_revertable(gas_meter)),
            Err(e) => {
                tracing::debug!(
                    sender = %context.sender(),
                    error = ?e,
                    "Unable to reserve gas from sender"
                );
                Err(state_checkpoint)
            },
        }
    }

    /// Refunds any remaining gas to the payer after the transaction is processed.
    fn refund_remaining_gas(
        &self,
        tx: &Self::Tx,
        context: &Context<S>,
        gas_meter: &sov_modules_api::TxGasMeter<S::Gas>,
        state_checkpoint: &mut StateCheckpoint<S>,
    ) {
        self.bank.refund_remaining_gas(
            tx,
            gas_meter,
            context.sender(),
            &self.prover_incentives.id().to_payable(),
            &self.sequencer_registry.id().to_payable(),
            state_checkpoint,
        );
    }
}

impl<S: Spec, Da: DaSpec> SequencerAuthorization<S, Da> for Runtime<S, Da> {
    type SequencerStakeMeter = SequencerStakeMeter<S::Gas>;

    fn authorize_sequencer(
        &self,
        sequencer: &<Da as DaSpec>::Address,
        base_fee_per_gas: &<S::Gas as Gas>::Price,
        state_checkpoint: &mut StateCheckpoint<S>,
    ) -> Result<SequencerStakeMeter<S::Gas>, anyhow::Error> {
        self.sequencer_registry
            .authorize_sequencer(sequencer, base_fee_per_gas, state_checkpoint)
            .context("An error occurred while checking the sequencer bond")
    }

    fn penalize_sequencer(
        &self,
        sequencer: &Da::Address,
        sequencer_stake_meter: SequencerStakeMeter<S::Gas>,
        state_checkpoint: &mut StateCheckpoint<S>,
    ) {
        self.sequencer_registry.penalize_sequencer(
            sequencer,
            sequencer_stake_meter,
            state_checkpoint,
        );
    }

    fn refund_sequencer(
        &self,
        sequencer_stake_meter: &mut Self::SequencerStakeMeter,
        refund_amount: u64,
    ) {
        self.sequencer_registry
            .refund_sequencer(sequencer_stake_meter, refund_amount);
    }
}

impl<S: Spec, Da: DaSpec> RuntimeAuthorization<S, Da> for Runtime<S, Da> {
    type Tx = AuthenticatedTransactionData<S>;

    /// Prevents duplicate transactions from running.
    // TODO(@preston-evans98): Use type system to prevent writing to the `StateCheckpoint` during
    // this check
    fn check_uniqueness(
        &self,
        tx: &Self::Tx,
        _context: &Context<S>,
        state_checkpoint: &mut StateCheckpoint<S>,
    ) -> Result<(), anyhow::Error> {
        self.accounts.check_uniqueness(tx, state_checkpoint)
    }

    /// Marks a transaction as having been executed, preventing it from executing again.
    fn mark_tx_attempted(
        &self,
        tx: &Self::Tx,
        _sequencer: &Da::Address,
        state_checkpoint: &mut StateCheckpoint<S>,
    ) {
        self.accounts.mark_tx_attempted(tx, state_checkpoint);
    }

    /// Resolves the context for a transaction.
    fn resolve_context(
        &self,
        tx: &Self::Tx,
        sequencer: &Da::Address,
        height: u64,
        working_set: &mut StateCheckpoint<S>,
    ) -> Result<Context<S>, anyhow::Error> {
        // TODO(@preston-evans98): This is a temporary hack to get the sequencer address
        // This should be resolved by the sequencer registry during blob selection
        let sequencer = self
            .sequencer_registry
            .resolve_da_address(sequencer, working_set)
            .ok_or(anyhow::anyhow!("Sequencer was no longer registered by the time of context resolution. This is a bug")).unwrap();
        let sender = self.accounts.resolve_sender_address(tx, working_set)?;
        Ok(Context::new(sender, sequencer, height))
    }
}
