use sov_modules_api::Spec;

use crate::{delegate::Eviction, Power, Relayer};

#[derive(
    Clone,
    Debug,
    PartialEq,
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(
    bound = "S::Address: serde::Serialize + serde::de::DeserializeOwned",
    rename_all = "snake_case"
)]
pub enum Event<S: Spec> {
    // Campaign
    CampaignInitialized {
        campaign_id: u64,
        campaigner: S::Address,
        evictions: Vec<Eviction<S>>,
    },
    CampaignIndexing {
        campaign_id: u64,
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
    SegmentPosted {
        campaign_id: u64,
        indexer: S::Address,
    },

    // Indexer
    IndexerRegistered {
        addr: S::Address,
        alias: String,
        sender: S::Address,
    },
    IndexerUnregistered {
        addr: S::Address,
        sender: S::Address,
    },

    // Relayer
    RelayerRegistered {
        addr: S::Address,
        sender: S::Address,
    },
    RelayerUnregistered {
        addr: S::Address,
        sender: S::Address,
    },

    // Voting
    VotingPowerUpdated {
        addr: S::Address,
        power: Power,
        relayer: Relayer<S>,
    },
}
