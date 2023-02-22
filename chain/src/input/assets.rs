use serde::{Deserialize, Serialize};

use crate::{Address, Amount};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Transfer {
    pub from: Address,
    pub to: Address,
    pub denom: String,
    pub amount: Amount,
}
