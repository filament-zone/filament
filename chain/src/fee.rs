use serde::{Deserialize, Serialize};

use crate::{asset, Amount};

// TODO(xla): Fill out and document.
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Fee {
    asset_id: asset::Id,
    amount: Amount,
}
