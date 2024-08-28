use sov_modules_api::Spec;

use crate::{campaign::Payment, delegate::Eviction};

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
        campaigner: S::Address,
        id: u64,
    },
    CampaignInitialized {
        id: u64,
        campaigner: S::Address,
        payment: Option<Payment>,
        evictions: Vec<Eviction<S>>,
    },

    CampaignIndexing {
        id: u64,
        indexer: S::Address,
    },

    CriteriaProposed {
        campaign_id: u64,
        proposer: S::Address,
        proposal_id: u64,
    },

    CriteriaConfirmed {
        campaign_id: u64,
        proposal_id: Option<u64>,
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
