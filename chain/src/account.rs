use cosmrs::AccountId;
use ed25519_consensus::VerificationKey;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Account {
    address: AccountId,
    verification_key: VerificationKey,

    account_number: u64,
    sequence: u64,
}
