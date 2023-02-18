use bech32::{self, FromBase32, ToBase32, Variant};
use pulzaar_crypto::{Address, VerificationKey};

use crate::Error;

pub const BECH32_ADDRESS_PREFIX: &str = "plzaddr";

pub trait ToBech32 {
    type Err;

    fn to_bech32(&self) -> Result<String, Self::Err>;
}

impl ToBech32 for Address {
    type Err = Error;

    fn to_bech32(&self) -> Result<String, Self::Err> {
        bech32::encode(
            BECH32_ADDRESS_PREFIX,
            self.0.as_bytes().to_base32(),
            Variant::Bech32m,
        )
        .map_err(Error::from)
    }
}

pub trait FromBech32: Sized {
    type Err;

    fn from_bech32<S>(s: S) -> Result<Self, Self::Err>
    where
        S: Into<String>;
}

impl FromBech32 for Address {
    type Err = Error;

    fn from_bech32<S>(address: S) -> Result<Self, Self::Err>
    where
        S: Into<String>,
    {
        let (hrp, data, variant) = bech32::decode(&address.into())?;

        if hrp != BECH32_ADDRESS_PREFIX {
            return Err(Error::Bech32UnexpectedPrefix);
        }

        if variant != Variant::Bech32m {
            return Err(Error::Bech32UnexpectedVariant);
        }

        let data = Vec::<u8>::from_base32(&data)?;
        let vk = VerificationKey::try_from(data.as_slice())?;

        Ok(Address(vk))
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use pulzaar_crypto::{SigningKey, VerificationKey};
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
