use pulzaar_crypto::Address;
use serde::{Deserialize, Serialize};

use crate::Funds;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Transfer {
    from: Address,
    to: Address,
    funds: Funds,
}
