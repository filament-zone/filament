use sov_modules_api::Spec;

use crate::campaign::ChainId;

#[derive(
    Clone,
    Debug,
    PartialEq,
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum Event<S: Spec> {
    CampaignCreated {
        id: u64,

        origin: ChainId,
        origin_id: u64,
    },

    CampaignIndexing {
        id: u64,
        indexer: S::Address,
    },

    IndexerRegistered {
        addr: S::Address,
        alias: String,
    },
    IndexerUnregistered {
        addr: S::Address,
    },

    SegmentPosted {
        id: u64,
        indexer: S::Address,
    },
}
