use serde::{Deserialize, Serialize};

use crate::{fee::Fee, input::Input};

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
    // List of intents carried by the transaction.
    inputs: Vec<Input>,

    // Intended chain for the transaction to land on, to be included to prevent replays on other
    // chains.
    chain_id: String,
    // Maximum height until the transaction is valid, doesn't expire if the value is zero.
    max_height: u64,

    // Fees of the transaction.
    // TODO(xla): Does this belong in the body?
    fee: Fee,
}
