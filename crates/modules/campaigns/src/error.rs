use sov_modules_api::Spec;
use thiserror::Error;

use crate::campaign::{ChainId, Status};

#[derive(Debug, Eq, PartialEq, Error)]
pub enum CampaignsError<S: Spec> {
    #[error("Campaign '{origin}-{origin_id}' exists")]
    CampaignExists { origin: ChainId, origin_id: u64 },

    #[error("Campaign '{id}' not found")]
    CampaignNotFound { id: u64 },

    #[error("Campaign id already exists. This is a bug - the id should be correctly incremented")]
    IdExists { id: u64 },

    #[error("Sender '{sender}' is not the registered indexer '{indexer}' for campaign '{id}'")]
    IndexerMissmatch {
        id: u64,
        indexer: S::Address,
        sender: S::Address,
    },

    #[error("Indexer '{indexer}' is not registered")]
    IndexerNotRegistered { indexer: S::Address },

    #[error("Invalid campaign status transition  attempted for '{id}' from '{current:?}' to '{attempted:?}'")]
    InvalidTransition {
        id: u64,
        current: Status,
        attempted: Status,
    },

    #[error("Module nex_id is not set. This is a bug - the id should be set at genesis")]
    NextIdMissing,

    #[error("Segment for '{id}' exists")]
    SegmentExists { id: u64 },
}
