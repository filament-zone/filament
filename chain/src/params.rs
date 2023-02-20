use serde::{Deserialize, Serialize};

use crate::ChainId;

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct ChainParameters {
    pub chain_id: ChainId,
    pub epoch_duration: u64,
}
