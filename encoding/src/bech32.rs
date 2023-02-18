use bech32::{self, FromBase32, ToBase32, Variant};
use pulzaar_crypto::{Address, SigningKey};

use crate::Error;

const BECH32_ADDRESS_PREFIX: &str = "plzaddr";
const BECH32_SIGNKEY_PREFIX: &str = "plzkey";

pub trait ToBech32: AsRef<[u8]> {
    const PREFIX: &'static str;
    fn to_bech32(&self) -> Result<String, Error> {
        bech32::encode(Self::PREFIX, self.as_ref().to_base32(), Variant::Bech32m)
            .map_err(Error::from)
    }
}

pub trait FromBech32: Sized + for<'a> TryFrom<&'a [u8]> {
    const PREFIX: &'static str;

    fn from_bech32<S>(s: S) -> Result<Self, Error>
    where
        S: Into<String>,
    {
        let (hrp, data, variant) = bech32::decode(&s.into())?;

        if hrp != Self::PREFIX {
            return Err(Error::Bech32UnexpectedPrefix);
        }

        if variant != Variant::Bech32m {
            return Err(Error::Bech32UnexpectedVariant);
        }

        let data = Vec::<u8>::from_base32(&data)?;
        let data = data.as_slice();

        Self::try_from(data).map_err(|_err| Error::Bech32Conversion)
    }
}

impl ToBech32 for Address {
    const PREFIX: &'static str = BECH32_ADDRESS_PREFIX;
}

impl FromBech32 for Address {
    const PREFIX: &'static str = BECH32_ADDRESS_PREFIX;
}

impl ToBech32 for SigningKey {
    const PREFIX: &'static str = BECH32_SIGNKEY_PREFIX;
}

impl FromBech32 for SigningKey {
    const PREFIX: &'static str = BECH32_SIGNKEY_PREFIX;
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

    #[test]
    fn signing_key_bech32_roundtrip() {
        let sk = SigningKey::new(thread_rng());
        let encoded = sk.to_bech32().unwrap();
        let decoded = SigningKey::from_bech32(encoded).unwrap();
        assert_eq!(sk.to_bytes(), decoded.to_bytes());
    }
}
