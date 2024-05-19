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

    #[error("CampaignNoLongerRunning")]
    CampaignNoLongerRunning {},

    #[error("CampaignDeadlinePassed")]
    CampaignDeadlinePassed {},

    #[error("CampaignCannotDisperseFees")]
    CampaignCannotDisperseFees {},

    #[error("CampaignAllFundsPledged")]
    CampaignAllFundsPledged {},

    #[error("CampaignNotAllConversionsRegistered")]
    CampaignNotAllConversionsRegistered {},

    #[error("CampaignNotFinished")]
    CampaignNotFinished {},

    #[error("CampaignCanceled")]
    CampaignCanceled {},

    #[error("ConversionAlreadyRegistered")]
    ConversionAlreadyRegistered {},

    #[error("ConversionAlreadyClaimed")]
    ConversionAlreadyClaimed {},

    #[error("AllConversionsRegistered")]
    AllConversionsRegistered {},
}
