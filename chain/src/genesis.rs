use serde::{Deserialize, Serialize};

use crate::{Address, Amount, ChainParameters};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct AppState {
    pub allocations: Vec<Allocation>,
    pub chain_parameters: ChainParameters,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Allocation {
    pub address: Address,
    pub denom: String,
    pub amount: Amount,
}
