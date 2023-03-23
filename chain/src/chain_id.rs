use serde::{Deserialize, Serialize};

/// Typically a human readable string that identifies a chain and version but
/// there are no strict format requirements.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct ChainId(String);

impl TryFrom<String> for ChainId {
    type Error = eyre::Report;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(Self(value))
    }
}

impl TryFrom<&str> for ChainId {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_owned())
    }
}
