use serde::{Deserialize, Serialize};

use crate::{Address, Funds};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Transfer {
    pub from: Address,
    pub to: Address,
    pub funds: Funds,
}
