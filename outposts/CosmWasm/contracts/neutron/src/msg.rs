use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

use crate::state::{
    Campaign,
    CampaignBudget,
    CampaignStatus,
    Config,
    ConversionDesc,
    PayoutMechanism,
    SegmentDesc,
};

#[cw_serde]
pub struct InstantiateMsg {
    pub chain: String,
    pub controller: Addr,
    pub oracle: Addr,
    pub fee_recipient: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {
    CreateCampaignMsg {
        admin: Addr,
        indexer: Addr,
        attester: Addr,
        segment_desc: SegmentDesc,
        conversion_desc: ConversionDesc,
        payout_mech: PayoutMechanism,
        ends_at: u64,
    },
    FundCampaignMsg {
        id: u64,
        budget: CampaignBudget,
    },
    SetStatusIndexingMsg {
        id: u64,
    },

    RegisterSegmentMsg {
        id: u64,
        size: u64,
    },
    RegisterConversionsMsg {
        id: u64,
        who: Addr,
        amount: u128,
    },
    FinalizeCampaignMsg {
        id: u64,
    },

    ClaimIncentivesMsg {
        id: u64,
    },

    AbortCampaignMsg {
        id: u64,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    GetConfig {},

    #[returns(QueryCampaignsResponse)]
    QueryCampaignsByStatus {
        start: Option<u64>,
        limit: Option<u32>,
        status: CampaignStatus,
    },

    #[returns(GetCampaignResponse)]
    GetCampaign { id: u64 },
}

#[cw_serde]
pub struct QueryCampaignsResponse {
    pub campaigns: Vec<Campaign>,
}

#[cw_serde]
pub struct GetCampaignResponse {
    pub campaign: Campaign,
}

#[cw_serde]
pub struct MigrateMsg {}
