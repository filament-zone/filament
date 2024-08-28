use sov_modules_api::{CallResponse, Context, Spec, TxState};

use crate::{
    campaign::Payment,
    criteria::Criteria,
    delegate::Eviction,
    playbook::Budget,
    segment::Segment,
    Core,
    CoreError,
};

/// This enumeration represents the available call messages for interacting with
/// the `Core` module.
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
    Init {
        criteria: Criteria,
        budget: Budget,
        payment: Option<Payment>,
        evictions: Vec<Eviction<S>>,
    },

    ProposeCriteria {
        campaign_id: u64,
        criteria: Criteria,
    },

    ConfirmCriteria {
        campaign_id: u64,
        proposal_id: Option<u64>,
    },

    RejectCriteria {
        id: u64,
    },

    IndexCampaign {
        id: u64,
    },

    PostSegment {
        id: u64,
        segment: Segment,
    },

    RegisterIndexer(S::Address, String),
    UnregisterIndexer(S::Address),
}

impl<S: Spec> Core<S> {
    pub(crate) fn call_init_campaign(
        &self,
        criteria: Criteria,
        budget: Budget,
        payment: Option<Payment>,
        evictions: Vec<Eviction<S>>,
        context: &Context<S>,
        working_set: &mut impl TxState<S>,
    ) -> Result<CallResponse, CoreError<S>> {
        self.init_campaign(
            context.sender().clone(),
            criteria,
            budget,
            payment,
            evictions,
            working_set,
        )?;
        Ok(CallResponse::default())
    }

    pub(crate) fn call_propose_criteria(
        &self,
        id: u64,
        criteria: Criteria,
        context: &Context<S>,
        working_set: &mut impl TxState<S>,
    ) -> Result<CallResponse, CoreError<S>> {
        self.propose_criteria(context.sender().clone(), id, criteria, working_set)?;
        Ok(CallResponse::default())
    }

    pub(crate) fn call_confirm_criteria(
        &self,
        id: u64,
        proposal_id: Option<u64>,
        context: &Context<S>,
        working_set: &mut impl TxState<S>,
    ) -> Result<CallResponse, CoreError<S>> {
        self.confirm_criteria(context.sender().clone(), id, proposal_id, working_set)?;
        Ok(CallResponse::default())
    }

    pub(crate) fn call_reject_criteria(
        &self,
        id: u64,
        context: &Context<S>,
        working_set: &mut impl TxState<S>,
    ) -> Result<CallResponse, CoreError<S>> {
        self.reject_criteria(context.sender().clone(), id, working_set)?;
        Ok(CallResponse::default())
    }

    pub(crate) fn call_index_campaign(
        &self,
        id: u64,
        context: &Context<S>,
        working_set: &mut impl TxState<S>,
    ) -> Result<CallResponse, CoreError<S>> {
        self.index_campaign(context.sender().clone(), id, working_set)?;
        Ok(CallResponse::default())
    }

    pub(crate) fn call_post_segment(
        &self,
        id: u64,
        segment: Segment,
        context: &Context<S>,
        working_set: &mut impl TxState<S>,
    ) -> Result<CallResponse, CoreError<S>> {
        self.post_segment(context.sender().clone(), id, segment, working_set)?;
        Ok(CallResponse::default())
    }

    pub(crate) fn call_register_indexer(
        &self,
        indexer: S::Address,
        alias: String,
        context: &Context<S>,
        working_set: &mut impl TxState<S>,
    ) -> Result<CallResponse, CoreError<S>> {
        self.register_indexer(context.sender().clone(), indexer, alias, working_set)?;
        Ok(CallResponse::default())
    }

    pub(crate) fn call_unregister_indexer(
        &self,
        indexer: S::Address,
        context: &Context<S>,
        working_set: &mut impl TxState<S>,
    ) -> Result<CallResponse, CoreError<S>> {
        self.unregister_indexer(context.sender().clone(), indexer, working_set)?;
        Ok(CallResponse::default())
    }
}
