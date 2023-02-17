use serde::{Deserialize, Serialize};

use crate::{asset, Amount};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Funds {
    asset_id: asset::Id,
    amount: Amount,
}
