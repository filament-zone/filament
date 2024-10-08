use cosmwasm_std::{Addr, BankMsg, Coin, DepsMut, Env, MessageInfo, Order, Response, Uint128};

use super::query::load_campaign;
use crate::{
    error::ContractError,
    state::{
        Campaign,
        CampaignBudget,
        CampaignStatus,
        ConversionDesc,
        PayoutMechanism,
        SegmentDesc,
        ATTESTING_CAMPAIGNS,
        CAMPAIGN_ID,
        CANCELED_CAMPAIGNS,
        CONF,
        CONVERSIONS,
        CREATED_CAMPAIGNS,
        FAILED_CAMPAIGNS,
        FINISHED_CAMPAIGNS,
        FUNDED_CAMPAIGNS,
        INDEXING_CAMPAIGNS,
    },
};

#[allow(clippy::too_many_arguments)]
pub fn create_campaign(
    deps: DepsMut<'_>,
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
    Ok(Response::new().add_attributes(vec![
        ("campaign_id", id.to_string()),
        ("campaign_status", c.status.to_string()),
    ]))
}

pub fn is_oracle(deps: &DepsMut<'_>, info: &MessageInfo) -> bool {
    let conf = CONF.load(deps.storage);

    conf.is_ok_and(|c| c.oracle == info.sender)
}

// Funding a campaign moves it automatically from `Created` to `Indexing` because
// there is currently no intermediate step on the hub where indexers bid for
// execution.
// The creator needs to supply enough funds to cover fee and incentives.
pub fn fund_campaign(
    deps: DepsMut<'_>,
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
    campaign.budget = Some(budget);

    CREATED_CAMPAIGNS.remove(deps.storage, id);
    FUNDED_CAMPAIGNS.save(deps.storage, id, &campaign)?;

    Ok(Response::new().add_attribute("campaign_id", id.to_string()))
}

pub fn set_status_indexing(
    deps: DepsMut<'_>,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    if !is_oracle(&deps, &info) {
        return Err(ContractError::Unauthorized {});
    }

    let mut campaign = FUNDED_CAMPAIGNS
        .load(deps.storage, id)
        .map_err(|_| ContractError::CampaignIdNotFound(id))?;

    campaign.status = CampaignStatus::Indexing;

    FUNDED_CAMPAIGNS.remove(deps.storage, id);
    INDEXING_CAMPAIGNS.save(deps.storage, id, &campaign)?;

    Ok(Response::new().add_attributes(vec![
        ("campaign_id", id.to_string()),
        ("campaign_status", campaign.status.to_string()),
    ]))
}

pub fn register_segment(
    deps: DepsMut<'_>,
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

    Ok(Response::new().add_attributes(vec![
        ("campaign_id", id.to_string()),
        ("campaign_status", campaign.status.to_string()),
    ]))
}

pub fn register_conversion(
    deps: DepsMut<'_>,
    env: Env,
    info: MessageInfo,
    id: u64,
    who: Addr,
    amount: u128,
) -> Result<Response, ContractError> {
    if !is_oracle(&deps, &info) {
        return Err(ContractError::Unauthorized {});
    }

    let mut campaign = ATTESTING_CAMPAIGNS.load(deps.storage, id)?;

    if !campaign.is_running() {
        return Err(ContractError::CampaignNoLongerRunning {});
    }

    if campaign.is_beyond_deadline(env.block.time.nanos()) {
        return Err(ContractError::CampaignDeadlinePassed {});
    }

    if CONVERSIONS.has(deps.storage, (id, who.clone())) {
        return Err(ContractError::ConversionAlreadyRegistered {});
    }

    let cs: u64 = CONVERSIONS
        .prefix(id)
        .range(deps.storage, None, None, Order::Ascending)
        .count()
        .try_into()
        .unwrap();

    if cs == campaign.segment_size {
        return Err(ContractError::AllConversionsRegistered {});
    }

    if campaign.budget_left() == 0 || campaign.budget_left() < amount {
        return Err(ContractError::CampaignAllFundsPledged {});
    }

    CONVERSIONS.save(deps.storage, (id, who), &(amount, false))?;

    campaign.spent += amount;

    ATTESTING_CAMPAIGNS.save(deps.storage, id, &campaign)?;

    Ok(Response::new().add_attributes(vec![("campaign_id", id.to_string())]))
}

// Anyone can finalize a campaign given it is in the correct state:
//   - in Attesting state
//   AND
//   - all conversions are posted (count(conversion) == segment_size)
//   - or we are beyond the deadline (block.time > ends_at)
// If that is the case, then the campaign moves from Attesting to Finalized
// and fees are dispersed to the attester, indexer and protocol.
pub fn finalized_campaign(
    deps: DepsMut<'_>,
    env: Env,
    _info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let mut campaign = ATTESTING_CAMPAIGNS.load(deps.storage, id)?;

    let cs: u64 = CONVERSIONS
        .prefix(id)
        .range(deps.storage, None, None, Order::Ascending)
        .count()
        .try_into()
        .unwrap();

    if cs != campaign.segment_size && !campaign.is_beyond_deadline(env.block.time.nanos()) {
        return Err(ContractError::CampaignNotAllConversionsRegistered {});
    }

    let mut msgs = vec![];
    if !campaign.fee_claimed {
        msgs.append(fee_msgs(&deps, &campaign)?.as_mut())
    }

    campaign.fee_claimed = true;
    campaign.status = CampaignStatus::Finished;

    ATTESTING_CAMPAIGNS.remove(deps.storage, id);
    FINISHED_CAMPAIGNS.save(deps.storage, id, &campaign)?;

    Ok(Response::new().add_messages(msgs))
}

// XXX: currently just splitting the fees 40/40/20 between attester, indexer
//      and protocol
pub fn fee_msgs(deps: &DepsMut<'_>, campaign: &Campaign) -> Result<Vec<BankMsg>, ContractError> {
    let fees = campaign
        .budget
        .clone()
        .ok_or(ContractError::CampaignCannotDisperseFees {})?
        .fee;
    let out = fees.amount / Uint128::from(5_u128);

    let att_out = out * Uint128::from(2_u128);
    let ind_out = out * Uint128::from(2_u128);
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

    Ok(vec![att, ind, pro])
}

pub fn claim_incentives(
    deps: DepsMut<'_>,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let campaign = load_campaign(deps.as_ref(), id)?;

    if campaign.is_running() {
        return Err(ContractError::CampaignNotFinished {});
    }

    if campaign.status == CampaignStatus::Canceled {
        return Err(ContractError::CampaignCanceled {});
    }

    let (payout, claimed) = CONVERSIONS.load(deps.storage, (id, info.sender.clone()))?;

    if claimed {
        return Err(ContractError::ConversionAlreadyClaimed {});
    }
    // How did we even get here if there is no budget at this point?
    let out_coin = campaign
        .budget
        .ok_or(ContractError::CampaignCannotPayout {})?
        .incentives;

    let msg = BankMsg::Send {
        to_address: info.sender.to_string(),
        amount: vec![Coin {
            denom: out_coin.denom,
            amount: Uint128::from(payout),
        }],
    };
    CONVERSIONS.save(deps.storage, (id, info.sender.clone()), &(payout, true))?;

    Ok(Response::new().add_message(msg))
}

pub fn abort_campaign(
    deps: DepsMut<'_>,
    info: MessageInfo,
    id: u64,
) -> Result<Response, ContractError> {
    let mut campaign = load_campaign(deps.as_ref(), id)?;

    if !campaign.is_admin(&info.sender) {
        return Err(ContractError::Unauthorized {});
    }

    // XXX: right now this means that once all conversions are registered,
    //      the campaigner can no longer get a refund
    let mut msgs = vec![];
    if campaign.has_budget() {
        // if we have a budget the unwrap should not fail
        let out_coin = campaign.budget.clone().unwrap().incentives;
        let payout = campaign.budget_left();

        let snd = BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![Coin {
                denom: out_coin.denom,
                amount: Uint128::from(payout),
            }],
        };

        msgs.push(snd);
    }

    let status = campaign.status;
    campaign.status = CampaignStatus::Canceled;

    CANCELED_CAMPAIGNS.save(deps.storage, id, &campaign)?;
    remove_campaign(deps, id, status);

    Ok(Response::new().add_messages(msgs).add_attributes(vec![
        ("campaign_id", id.to_string()),
        ("campaign_status", campaign.status.to_string()),
    ]))
}

pub fn remove_campaign(deps: DepsMut<'_>, id: u64, status: CampaignStatus) {
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
