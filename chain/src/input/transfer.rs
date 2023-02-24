use serde::{Deserialize, Serialize};

use crate::{Address, Amount, Denom};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Transfer {
    pub from: Address,
    pub to: Address,
    pub denom: Denom,
    pub amount: Amount,
}
