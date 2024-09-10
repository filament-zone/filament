use std::fmt;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin, Uint128};
use cw_storage_plus::{Item, Map};
use serde::{de::Error, Deserialize, Deserializer};

#[cw_serde]
pub struct Config {
    /// Chain Id
    pub chain_id: String,
    pub controller: Addr,
    pub oracle: Addr,
    pub fee_recipient: Addr,
}

impl Config {
    pub fn is_controller(&self, a: Addr) -> bool {
        a == self.controller
    }
}

pub const CONF: Item<'_, Config> = Item::new("config");

pub const CAMPAIGN_ID: Item<'_, u64> = Item::new("campaign_id");

pub const CREATED_CAMPAIGNS: Map<'_, u64, Campaign> = Map::new("created_campaigns");
pub const FUNDED_CAMPAIGNS: Map<'_, u64, Campaign> = Map::new("funded_campaigns");
pub const INDEXING_CAMPAIGNS: Map<'_, u64, Campaign> = Map::new("indexing_campaigns");
pub const ATTESTING_CAMPAIGNS: Map<'_, u64, Campaign> = Map::new("attesting_campaigns");
pub const FINISHED_CAMPAIGNS: Map<'_, u64, Campaign> = Map::new("finished_campaigns");
pub const CANCELED_CAMPAIGNS: Map<'_, u64, Campaign> = Map::new("canceled_campaigns");
pub const FAILED_CAMPAIGNS: Map<'_, u64, Campaign> = Map::new("failed_campaigns");

pub const CONVERSIONS: Map<'_, (u64, Addr), (u128, bool)> = Map::new("campaign_conversions");

#[cw_serde]
pub enum CampaignStatus {
    Created,
    Funded,
    Indexing,
    Attesting,
    Finished,
    Canceled,
    Failed,
}

impl fmt::Display for CampaignStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn number_from_string<'de, D>(deserializer: D) -> Result<u128, D::Error>
where
    D: Deserializer<'de>,
{
    let input: String = Deserialize::deserialize(deserializer)?;
    input
        .parse::<u128>()
        .map_err(|e| D::Error::custom(format!("could not deserialize u128: {:}", e)))
}

#[cw_serde]
pub struct Campaign {
    pub id: u64,
    pub admin: Addr,
    pub status: CampaignStatus,
    pub budget: Option<CampaignBudget>,
    #[serde(deserialize_with = "number_from_string")]
    pub spent: u128,
    pub indexer: Addr,
    pub attester: Addr,
    pub segment_desc: SegmentDesc,
    pub segment_size: u64,
    pub conversion_desc: ConversionDesc,
    pub payout_mech: PayoutMechanism,
    pub ends_at: u64,
    pub fee_claimed: bool,
}

impl Campaign {
    pub fn is_admin(&self, who: &Addr) -> bool {
        self.admin == who
    }

    pub fn is_running(&self) -> bool {
        self.status != CampaignStatus::Canceled
            && self.status != CampaignStatus::Finished
            && self.status != CampaignStatus::Failed
    }

    pub fn has_budget(&self) -> bool {
        self.budget.is_some()
    }

    pub fn is_incentive_denom(&self, denom: String) -> bool {
        self.has_budget() && self.budget.clone().unwrap().incentives.denom == denom
    }

    pub fn is_beyond_deadline(&self, curr: u64) -> bool {
        self.ends_at > 0 && self.ends_at < curr
    }

    pub fn payout_amount(&self) -> Option<u128> {
        let budget = self.budget.clone()?;
        match self.payout_mech {
            PayoutMechanism::ProportionalPerConversion => {
                Some(budget.incentives.amount.u128() / self.segment_size as u128)
            },
        }
    }

    pub fn can_payout(&self) -> bool {
        let budget = self.budget.clone().unwrap_or(CampaignBudget {
            incentives: Coin {
                denom: "".to_string(),
                amount: Uint128::zero(),
            },
            fee: Coin {
                denom: "".to_string(),
                amount: Uint128::zero(),
            },
        });
        let out = self.payout_amount().unwrap_or(0);
        self.spent + out <= budget.incentives.amount.u128()
    }

    pub fn budget_left(&self) -> u128 {
        if !self.has_budget() {
            return 0;
        }

        let mut budget = self.budget.clone().unwrap().incentives;

        budget.amount -= Uint128::from(self.spent);

        budget.amount.u128()
    }
}

#[cw_serde]
pub struct CampaignBudget {
    pub fee: Coin,
    pub incentives: Coin,
}

#[cw_serde]
pub struct SegmentDesc {
    pub kind: SegmentKind,
    pub sources: Vec<String>,
    pub proof: SegmentProofMechanism,
}

#[cw_serde]
pub enum SegmentKind {
    GithubTopNContributors(u16),
    GithubAllContributors,
}

#[cw_serde]
pub enum SegmentProofMechanism {
    Ed25519Signature,
}

#[cw_serde]
pub struct ConversionDesc {
    pub kind: ConversionMechanism,
    pub proof: ConversionProofMechanism,
}

#[cw_serde]
pub enum ConversionMechanism {
    Social(Auth),
}

#[cw_serde]
pub enum ConversionProofMechanism {
    Ed25519Signature,
}

#[cw_serde]
pub enum Auth {
    Github,
}

#[cw_serde]
pub enum PayoutMechanism {
    ProportionalPerConversion,
}
