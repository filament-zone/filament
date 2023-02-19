use pulzaar_crypto::Address;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum Account {
    Single {
        address: Address,

        id: u64,
        sequence: u64,
    },
}
