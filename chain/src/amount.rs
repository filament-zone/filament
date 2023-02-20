use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct Amount(u128);

impl From<u128> for Amount {
    fn from(value: u128) -> Self {
        Self(value)
    }
}
