use pulzaar_crypto::{Signature, VerificationKey};
use serde::{Deserialize, Serialize};

use crate::input::Input;

/// A Pulzaar transactin.
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Transaction {
    auth: Auth,
    body: Body,
}

impl Transaction {
    pub fn inputs(&self) -> impl Iterator<Item = &Input> {
        self.body.inputs.iter()
    }
}

/// Authnetication information.
#[derive(Debug, PartialEq, Deserialize, Serialize)]
enum Auth {
    /// Single signature.
    Ed25519 {
        verification_key: VerificationKey,
        signature: Signature,
    },
}

/// Body of a Pulzaar transaction.
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Body {
    /// List of inputs carried by the transaction to advance the state machine.
    inputs: Vec<Input>,

    /// Intended chain for the transaction to land on, to be included to prevent replays on other
    /// chains.
    chain_id: String,
    /// Maximum height until the transaction is valid, doesn't expire if the value is zero.
    max_height: u64,

    /// Account id to match tx signers account.
    account_id: u64,
    /// Account sequence to match tx signers account state.
    sequence: u64,
}
