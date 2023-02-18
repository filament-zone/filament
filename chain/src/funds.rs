use serde::{Deserialize, Serialize};

use crate::{asset, Amount};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Funds {
    asset_id: asset::Id,
    amount: Amount,
}
