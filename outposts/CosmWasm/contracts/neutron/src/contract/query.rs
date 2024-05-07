use cosmwasm_std::Deps;

use crate::{
    msg::{GetCampaignResponse, QueryCampaignsResponse},
    state::{
        Campaign, CampaignStatus, Config, ATTESTING_CAMPAIGNS, CANCELED_CAMPAIGNS, CONF,
        CREATED_CAMPAIGNS, FAILED_CAMPAIGNS, FINISHED_CAMPAIGNS, FUNDED_CAMPAIGNS,
        INDEXING_CAMPAIGNS,
    },
    ContractError,
};

pub fn get_config(deps: Deps) -> Result<Config, ContractError> {
    let conf = CONF.load(deps.storage)?;

    Ok(conf)
}

pub fn get_campaign(deps: Deps, id: u64) -> Result<GetCampaignResponse, ContractError> {
    let campaign = load_campaign(deps, id)?;
    Ok(GetCampaignResponse { campaign })
}

// XXX
pub fn query_campaigns_by_status(
    _deps: Deps,
    _start: Option<u64>,
    _limit: Option<u32>,
    status: CampaignStatus,
) -> Result<QueryCampaignsResponse, ContractError> {
    let campaigns: Vec<Campaign> = match status {
        _ => vec![],
    };

    Ok(QueryCampaignsResponse { campaigns })
}

pub fn load_campaign(deps: Deps, id: u64) -> Result<Campaign, ContractError> {
    let c = if CREATED_CAMPAIGNS.has(deps.storage, id) {
        CREATED_CAMPAIGNS.load(deps.storage, id)?
    } else if FUNDED_CAMPAIGNS.has(deps.storage, id) {
        FUNDED_CAMPAIGNS.load(deps.storage, id)?
    } else if INDEXING_CAMPAIGNS.has(deps.storage, id) {
        INDEXING_CAMPAIGNS.load(deps.storage, id)?
    } else if ATTESTING_CAMPAIGNS.has(deps.storage, id) {
        ATTESTING_CAMPAIGNS.load(deps.storage, id)?
    } else if FINISHED_CAMPAIGNS.has(deps.storage, id) {
        FINISHED_CAMPAIGNS.load(deps.storage, id)?
    } else if FAILED_CAMPAIGNS.has(deps.storage, id) {
        FAILED_CAMPAIGNS.load(deps.storage, id)?
    } else if CANCELED_CAMPAIGNS.has(deps.storage, id) {
        CANCELED_CAMPAIGNS.load(deps.storage, id)?
    } else {
        return Err(ContractError::CampaignIdNotFound(id));
    };

    Ok(c)
}
