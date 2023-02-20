use serde::{Deserialize, Serialize};

use crate::ChainParameters;

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct AppState {
    pub chain_parameters: ChainParameters,
}
