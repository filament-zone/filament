use sov_modules_api::{CallResponse, Context, Spec, TxState};

use crate::{
    campaign::ChainId,
    error::CampaignsError,
    playbook::Playbook,
    segment::Segment,
    Campaigns,
};

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
    CreateCampaign {
        origin: String,
        origin_id: u64,

        indexer: S::Address,
        attester: S::Address,

        playbook: Playbook,
    },

    IndexCampaign {
        id: u64,
    },

    PostSegment {
        id: u64,
        segment: Segment,
    },
}

impl<S: Spec> Campaigns<S> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn call_create_campaign(
        &self,
        origin: ChainId,
        origin_id: u64,
        indexer: S::Address,
        attester: S::Address,
        playbook: Playbook,
        context: &Context<S>,
        working_set: &mut impl TxState<S>,
    ) -> Result<CallResponse, CampaignsError<S>> {
        self.create_campaign(
            context.sender().clone(),
            origin_id,
            origin,
            indexer,
            attester,
            playbook,
            working_set,
        )?;
        Ok(CallResponse::default())
    }

    pub(crate) fn call_index_campaign(
        &self,
        id: u64,
        context: &Context<S>,
        working_set: &mut impl TxState<S>,
    ) -> Result<CallResponse, CampaignsError<S>> {
        self.index_campaign(context.sender().clone(), id, working_set)?;
        Ok(CallResponse::default())
    }

    pub(crate) fn call_post_segment(
        &self,
        id: u64,
        segment: Segment,
        context: &Context<S>,
        working_set: &mut impl TxState<S>,
    ) -> Result<CallResponse, CampaignsError<S>> {
        self.post_segment(context.sender().clone(), id, segment, working_set)?;
        Ok(CallResponse::default())
    }
}
