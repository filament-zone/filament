use serde::{Deserialize, Serialize};

use crate::input::Input;

/// A Pulzaar transactin.
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Transaction {
    body: Body,
    // TODO(xla): Figure out signature schemes and layout.
}

impl Transaction {
    pub fn inputs(&self) -> impl Iterator<Item = &Input> {
        self.body.inputs.iter()
    }
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
