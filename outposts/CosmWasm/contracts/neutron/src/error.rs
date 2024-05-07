use cosmwasm_std::StdError;
use thiserror::Error;

use crate::state::CampaignStatus;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Campaign Id {0} not found")]
    CampaignIdNotFound(u64),

    #[error("NoFundsProvided")]
    NoFundsProvided {},

    #[error("FundsBudgetMismatch")]
    FundsBudgetMismatch {},

    #[error("AlreadyFunded")]
    AlreadyFunded {},

    #[error("InvalidStateTransition to {0}")]
    InvalidStateTransition(CampaignStatus),

    #[error("CampaignNoFunds")]
    CampaignNoFunds {},

    #[error("CampaignCannotPayout")]
    CampaignCannotPayout {},

    #[error("CampaignCannotDisperseFees")]
    CampaignCannotDisperseFees {},
}
