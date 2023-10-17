use std::fmt::Display;

use filament_crypto::VerificationKey;
use filament_encoding::{FromBech32, ToBech32};
use serde::{Deserialize, Serialize};

const BECH32_ADDRESS_PREFIX: &str = "fltaddr";

/// A unique identifier for a state entry.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Address(VerificationKey);

impl Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in &self.0.to_bytes() {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

impl AsRef<[u8]> for Address {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl From<VerificationKey> for Address {
    fn from(vk: VerificationKey) -> Self {
        Self(vk)
    }
}

impl TryFrom<&[u8]> for Address {
    type Error = eyre::Report;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        VerificationKey::try_from(bytes)
            .map(Address)
            .map_err(|e| eyre::eyre!(e))
    }
}

impl ToBech32 for Address {
    const PREFIX: &'static str = BECH32_ADDRESS_PREFIX;
}

impl FromBech32 for Address {
    const PREFIX: &'static str = BECH32_ADDRESS_PREFIX;
}

#[cfg(test)]
mod tests {
    use filament_crypto::{SigningKey, VerificationKey};
    use pretty_assertions::assert_eq;
    use rand::thread_rng;

    use super::*;

    #[test]
    fn address_bech32_roundtrip() {
        let address = Address(VerificationKey::from(&SigningKey::new(thread_rng())));
        let encoded = address.to_bech32().unwrap();
        let decoded = Address::from_bech32(encoded).unwrap();
        assert_eq!(address, decoded);
    }
}
