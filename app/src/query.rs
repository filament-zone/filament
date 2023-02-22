use std::fmt::Display;

use eyre::Report;

#[derive(Clone, Debug, PartialEq)]
pub enum Prefix {
    Accounts,
}

impl Display for Prefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = match self {
            Self::Accounts => "/accounts",
        };

        write!(f, "{}", prefix)
    }
}

impl TryFrom<&str> for Prefix {
    type Error = Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "/accounts" => Ok(Self::Accounts),
            _ => Err(eyre::eyre!("unsupported query prefix")),
        }
    }
}
