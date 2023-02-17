use cosmrs::AccountId;
use serde::{Deserialize, Serialize};

use crate::Funds;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Transfer {
    from: AccountId,
    to: AccountId,
    funds: Funds,
}
