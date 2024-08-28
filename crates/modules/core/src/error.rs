use sov_modules_api::Spec;
use thiserror::Error;

use crate::campaign::{ChainId, Phase};

#[derive(Debug, Eq, PartialEq, Error)]
pub enum CoreError<S: Spec> {
    #[error("Module admin is not set. This is a bug - the admin should be set at genesis")]
    AdminNotSet,

    #[error("Campaign '{origin}-{origin_id}' exists")]
    CampaignExists { origin: ChainId, origin_id: u64 },

    #[error("Campaign '{id}' not found")]
    CampaignNotFound { id: u64 },

    #[error("Campaign id already exists. This is a bug - the id should be correctly incremented")]
    IdExists { id: u64 },

    #[error("Invalid criteria proposal, campaign is not in criteria phase")]
    InvalidCriteriaProposal { campaign_id: u64 },

    #[error("Sender '{sender}' is not the registered indexer '{indexer:?}' for campaign '{id}'")]
    IndexerMissmatch {
        id: u64,
        indexer: Option<S::Address>,
        sender: S::Address,
    },

    #[error("Indexer '{indexer}' is not registered")]
    IndexerNotRegistered { indexer: S::Address },

    #[error("Invalid eviction, only proposed delegates can be evicted")]
    InvalidEviction,

    #[error("Invalid proposer: {reason}")]
    InvalidProposer { reason: String },

    #[error("Invalid campaign phase transition attempted for '{id}' from '{current:?}' to '{attempted:?}'")]
    InvalidTransition {
        id: u64,
        current: Phase,
        attempted: Phase,
    },

    #[error("Missing criteria")]
    MissingCriteria,

    #[error("Module nex_id is not set. This is a bug - the id should be set at genesis")]
    NextIdMissing,

    #[error("Segment for '{id}' exists")]
    SegmentExists { id: u64 },

    #[error("Sender '{sender}' is not an admin")]
    SenderNotAdmin { sender: S::Address },

    #[error("Sender '{sender}' is not the campaigner")]
    SenderNotCampaigner { sender: S::Address },
}
