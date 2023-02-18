use serde::{Deserialize, Serialize};

use crate::{Ed25519Error, VerificationKey};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Address(pub VerificationKey);

impl From<VerificationKey> for Address {
    fn from(vk: VerificationKey) -> Self {
        Self(vk)
    }
}

impl AsRef<[u8]> for Address {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl TryFrom<&[u8]> for Address {
    type Error = Ed25519Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        VerificationKey::try_from(bytes).map(Address)
    }
}
