use cosmwasm_std::{Addr, BankMsg, Coin, DepsMut, MessageInfo, Response, Uint128};

use crate::error::ContractError;
use crate::state::{
    Campaign, CampaignBudget, CampaignStatus, ConversionDesc, PayoutMechanism, SegmentDesc,
    ATTESTING_CAMPAIGNS, CAMPAIGN_ID, CANCELED_CAMPAIGNS, CONF, CREATED_CAMPAIGNS,
    FAILED_CAMPAIGNS, FINISHED_CAMPAIGNS, FUNDED_CAMPAIGNS, INDEXING_CAMPAIGNS,
};

use super::query::load_campaign;

pub fn create_campaign(
    deps: DepsMut,
    admin: Addr,
    indexer: Addr,
    attester: Addr,
    segment_desc: SegmentDesc,
    conversion_desc: ConversionDesc,
    payout_mech: PayoutMechanism,
    ends_at: u64,
) -> Result<Response, ContractError> {
    let id: u64 = CAMPAIGN_ID.load(deps.storage)?;
    CAMPAIGN_ID.save(deps.storage, &(id + 1))?;
    let c = Campaign {
        id,
        admin,
        status: CampaignStatus::Created,
        budget: None,
        spent: 0,
        indexer,
        attester,
        segment_desc,
        segment_size: 0,
        conversion_desc,
        payout_mech,
        ends_at,
        fee_claimed: false,
    };
    CREATED_CAMPAIGNS.save(deps.storage, id, &c)?;
    Ok(Response::new().add_attribute("campaign_id", id.to_string()))
}

pub fn is_oracle(deps: &DepsMut, info: &MessageInfo) -> bool {
    let conf = CONF.load(deps.storage);

    conf.is_ok_and(|c| c.oracle == info.sender)
}

// Funding a campaign moves it automatically from `Created` to `Indexing` because
// there is currently no intermediate step on the hub where indexers bid for
// execution.
// The creator needs to supply enough funds to cover fee and incentives.
pub fn fund_campaign(
    deps: DepsMut,
    info: MessageInfo,
    id: u64,
    budget: CampaignBudget,
) -> Result<Response, ContractError> {
    let funds = info
        .funds
        .first()
        .ok_or(ContractError::NoFundsProvided {})?;
    // XXX(pm): error on fee == 0? or other fee policy?
    let sum = budget.incentives.amount + budget.fee.amount;

    // XXX(pm): handle funds sent > budget specified?
    if funds.denom != budget.incentives.denom
        || funds.denom != budget.fee.denom
        || funds.amount != sum
    {
        return Err(ContractError::FundsBudgetMismatch {});
    }

    let mut campaign = CREATED_CAMPAIGNS
        .load(deps.storage, id)
        .map_err(|_| ContractError::CampaignIdNotFound(id))?;

    if !campaign.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // The above should already fail if the campaign has been funded.
    if campaign.has_budget() {
        return Err(ContractError::AlreadyFunded {});
    }

    campaign.status = CampaignStatus::Indexing;

    CREATED_CAMPAIGNS.remove(deps.storage, id);
    INDEXING_CAMPAIGNS.save(deps.storage, id, &campaign)?;

    Ok(Response::new().add_attribute("campaign_id", id.to_string()))
}

pub fn register_segment(
    deps: DepsMut,
    info: MessageInfo,
    id: u64,
    size: u64,
) -> Result<Response, ContractError> {
    if !is_oracle(&deps, &info) {
        return Err(ContractError::Unauthorized {});
    }

    let mut campaign = INDEXING_CAMPAIGNS.load(deps.storage, id)?;
    INDEXING_CAMPAIGNS.remove(deps.storage, id);

    campaign.segment_size = size;
    campaign.status = CampaignStatus::Attesting;

    ATTESTING_CAMPAIGNS.save(deps.storage, id, &campaign)?;

    Ok(Response::new().add_attribute("campaign_id", id.to_string()))
}

pub fn register_conversion(
    deps: DepsMut,
    info: MessageInfo,
    id: u64,
    who: Addr,
) -> Result<Response, ContractError> {
    if !is_oracle(&deps, &info) {
        return Err(ContractError::Unauthorized {});
    }

    let mut campaign = ATTESTING_CAMPAIGNS.load(deps.storage, id)?;

    if !campaign.can_payout() {
        return Err(ContractError::CampaignCannotPayout {});
    }

    let payout = campaign
        .payout_coin()
        .ok_or(ContractError::CampaignCannotPayout {})?;
    let snd = BankMsg::Send {
        to_address: who.to_string(),
        amount: vec![payout.clone()],
    };

    campaign.spent += payout.amount.u128();

    ATTESTING_CAMPAIGNS.save(deps.storage, id, &campaign)?;

    Ok(Response::new().add_message(snd))
}

pub fn disperse_fees(
    deps: DepsMut,
    _info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let mut campaign = load_campaign(deps.as_ref(), id)?;

    if campaign.is_running() || !campaign.has_budget() || campaign.fee_claimed {
        return Err(ContractError::CampaignCannotDisperseFees {});
    }

    let fees = campaign
        .budget
        .clone()
        .ok_or(ContractError::CampaignCannotDisperseFees {})?
        .fee;
    let out = fees.amount / Uint128::from(5 as u128);

    let att_out = out * Uint128::from(2 as u128);
    let ind_out = out * Uint128::from(2 as u128);
    let pro_out = fees.amount - att_out - ind_out;

    let att = BankMsg::Send {
        to_address: campaign.attester.to_string(),
        amount: vec![Coin {
            denom: fees.denom.clone(),
            amount: att_out,
        }],
    };

    let ind = BankMsg::Send {
        to_address: campaign.indexer.to_string(),
        amount: vec![Coin {
            denom: fees.denom.clone(),
            amount: ind_out,
        }],
    };

    let pro = BankMsg::Send {
        to_address: CONF.load(deps.storage)?.fee_recipient.to_string(),
        amount: vec![Coin {
            denom: fees.denom.clone(),
            amount: pro_out,
        }],
    };

    campaign.fee_claimed = true;
    write_campaign(deps, id, &campaign)?;

    Ok(Response::new().add_messages(vec![att, ind, pro]))
}

pub fn abort_campaign(
    deps: DepsMut,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let mut campaign = load_campaign(deps.as_ref(), id)?;

    if !campaign.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    let mut msgs = vec![];
    if campaign.has_budget() {
        // if we have a budget the unwrap should not fail
        let out_coin = campaign.payout_coin().unwrap();
        let payout = campaign
            .budget_left()
            .or(Some(Coin {
                denom: out_coin.denom,
                amount: Uint128::from(0 as u128),
            }))
            .unwrap();

        let snd = BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![payout],
        };

        msgs.push(snd);
    }

    let status = campaign.status;
    campaign.status = CampaignStatus::Canceled;

    CANCELED_CAMPAIGNS.save(deps.storage, id, &campaign)?;
    remove_campaign(deps, id, status);

    Ok(Response::new().add_messages(msgs))
}

pub fn write_campaign(deps: DepsMut, id: u64, campaign: &Campaign) -> Result<(), ContractError> {
    match campaign.status {
        CampaignStatus::Created => CREATED_CAMPAIGNS.save(deps.storage, id, campaign)?,
        CampaignStatus::Funded => FUNDED_CAMPAIGNS.save(deps.storage, id, campaign)?,
        CampaignStatus::Indexing => INDEXING_CAMPAIGNS.save(deps.storage, id, campaign)?,
        CampaignStatus::Attesting => ATTESTING_CAMPAIGNS.save(deps.storage, id, campaign)?,
        CampaignStatus::Finished => FINISHED_CAMPAIGNS.save(deps.storage, id, campaign)?,
        CampaignStatus::Canceled => CANCELED_CAMPAIGNS.save(deps.storage, id, campaign)?,
        CampaignStatus::Failed => FAILED_CAMPAIGNS.save(deps.storage, id, campaign)?,
    };

    Ok(())
}

pub fn remove_campaign(deps: DepsMut, id: u64, status: CampaignStatus) {
    match status {
        CampaignStatus::Created => CREATED_CAMPAIGNS.remove(deps.storage, id),
        CampaignStatus::Funded => FUNDED_CAMPAIGNS.remove(deps.storage, id),
        CampaignStatus::Indexing => INDEXING_CAMPAIGNS.remove(deps.storage, id),
        CampaignStatus::Attesting => ATTESTING_CAMPAIGNS.remove(deps.storage, id),
        CampaignStatus::Finished => FINISHED_CAMPAIGNS.remove(deps.storage, id),
        CampaignStatus::Canceled => CANCELED_CAMPAIGNS.remove(deps.storage, id),
        CampaignStatus::Failed => FAILED_CAMPAIGNS.remove(deps.storage, id),
    }
}
