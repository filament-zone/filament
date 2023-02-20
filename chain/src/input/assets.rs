use serde::{Deserialize, Serialize};

use crate::{Address, Funds};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Transfer {
    from: Address,
    to: Address,
    funds: Funds,
}
