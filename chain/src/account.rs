use serde::{Deserialize, Serialize};

use crate::Address;

/// Collection of different account types
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum Account {
    /// Account controlled by a single public key
    Single {
        /// The key controlling the account
        address: Address,

        /// System wide unique identifier
        id: u64,
        /// Account level nonce.
        sequence: u64,
    },
}

impl Account {
    pub fn address(&self) -> &Address {
        match self {
            Account::Single { address, .. } => address,
        }
    }

    pub fn id(&self) -> u64 {
        match self {
            Account::Single { id, .. } => *id,
        }
    }

    pub fn sequence(&self) -> u64 {
        match self {
            Account::Single { sequence, .. } => *sequence,
        }
    }
}
