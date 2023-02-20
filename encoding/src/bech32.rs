use bech32::{self, FromBase32, ToBase32, Variant};

use crate::Error;

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
