use serde::{Deserialize, Serialize};

use crate::Error;

pub fn from_bytes<'a, T>(bytes: &'a [u8]) -> Result<T, Error>
where
    T: Deserialize<'a>,
{
    bcs::from_bytes(bytes).map_err(Error::Bcs)
}

pub fn to_bytes<T>(value: &T) -> Result<Vec<u8>, Error>
where
    T: ?Sized + Serialize,
{
    bcs::to_bytes(value).map_err(Error::Bcs)
}
