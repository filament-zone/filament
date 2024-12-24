use sov_mock_zkvm::MockZkVerifier;
use sov_modules_api::{default_spec::DefaultSpec, execution_mode::Zk, Spec};

use crate::{delegate::Eviction, voting::VoteOption, Power, Relayer};

#[derive(
    Clone,
    Debug,
    PartialEq,
    borsh::BorshDeserialize,
    borsh::BorshSerialize,
    serde::Serialize,
    serde::Deserialize,
    ts_rs::TS,
)]
#[serde(
    bound = "S::Address: serde::Serialize + serde::de::DeserializeOwned",
    rename_all = "snake_case"
)]
#[ts(export, concrete(S = DefaultSpec<MockZkVerifier, MockZkVerifier, Zk>))]
#[ts(export_to = "../../../../bindings/Event.ts")]
pub enum Event<S: Spec> {
    // Campaign
    CampaignDrafted {
        campaign_id: u64,

        #[ts(type = "string")]
        campaigner: S::Address,
        #[ts(type = "Array<string>")]
        evictions: Vec<Eviction<S>>,
    },
    CampaignInitialized {
        campaign_id: u64,
    },
    CampaignIndexing {
        campaign_id: u64,
        #[ts(type = "string")]
        indexer: S::Address,
    },
    CriteriaProposed {
        campaign_id: u64,
        #[ts(type = "string")]
        proposer: S::Address,
        proposal_id: u64,
    },
    CriteriaVoted {
        campaign_id: u64,
        #[ts(type = "string")]
        delegate: S::Address,
        old_option: Option<VoteOption>,
        option: VoteOption,
    },
    CriteriaConfirmed {
        campaign_id: u64,
        proposal_id: Option<u64>,
    },
    SegmentPosted {
        campaign_id: u64,
        #[ts(type = "string")]
        indexer: S::Address,
    },

    // Indexer
    IndexerRegistered {
        #[ts(type = "string")]
        addr: S::Address,
        alias: String,
        #[ts(type = "string")]
        sender: S::Address,
    },
    IndexerUnregistered {
        #[ts(type = "string")]
        addr: S::Address,
        #[ts(type = "string")]
        sender: S::Address,
    },

    // Relayer
    RelayerRegistered {
        #[ts(type = "string")]
        addr: S::Address,
        #[ts(type = "string")]
        sender: S::Address,
    },
    RelayerUnregistered {
        #[ts(type = "string")]
        addr: S::Address,
        #[ts(type = "string")]
        sender: S::Address,
    },

    // Voting
    VotingPowerUpdated {
        #[ts(type = "string")]
        addr: S::Address,
        power: Power,
        #[ts(type = "string")]
        relayer: Relayer<S>,
    },
}
