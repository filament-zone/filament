use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct ChainId(String);

impl TryFrom<String> for ChainId {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self(value))
    }
}
