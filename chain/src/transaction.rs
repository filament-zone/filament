use pulzaar_crypto::{SignBytes, Signature, VerificationKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};

use crate::{input::Input, Address, ChainId};

/// A Pulzaar transaction.
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Transaction {
    /// Authentication data for the transaction body.
    pub auth: Auth,
    /// Transaction body with inputs and auxiliary data.
    pub body: Body,
}

impl Transaction {
    pub fn inputs(&self) -> impl Iterator<Item = &Input> {
        self.body.inputs.iter()
    }
}

/// Authentication information.
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum Auth {
    /// Single signature.
    Ed25519 {
        verification_key: VerificationKey,
        signature: Signature,
    },
}

impl From<&Auth> for Address {
    fn from(auth: &Auth) -> Self {
        match auth {
            Auth::Ed25519 {
                verification_key, ..
            } => Address::from(*verification_key),
        }
    }
}

/// Body of a Pulzaar transaction.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Body {
    /// List of inputs carried by the transaction to advance the state machine.
    // TODO(xla): Use a container that can't be constructed without at least one element. e.g: https://github.com/cloudhead/nonempty
    pub inputs: Vec<Input>,

    /// Intended chain for the transaction to land on, to be included to prevent replays on other
    /// chains.
    pub chain_id: ChainId,
    /// Maximum height until the transaction is valid.
    pub max_height: Option<u64>,

    /// Account id to match tx signers account.
    pub account_id: u64,
    /// Account sequence to match tx signers account state.
    pub sequence: u64,
}

impl SignBytes for Body {
    fn sign_bytes(&self) -> eyre::Result<Vec<u8>> {
        let mut hasher = Sha256::new();
        hasher.update(pulzaar_encoding::to_bytes(&self)?);
        Ok(hasher.finalize().to_vec())
    }
}
