use serde::{Deserialize, Serialize};

use crate::Address;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum Account {
    Single {
        address: Address,

        id: u64,
        sequence: u64,
    },
}
