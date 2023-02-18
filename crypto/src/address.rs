use serde::{Deserialize, Serialize};

use crate::VerificationKey;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Address(pub VerificationKey);

impl From<VerificationKey> for Address {
    fn from(vk: VerificationKey) -> Self {
        Self(vk)
    }
}
