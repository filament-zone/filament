use std::fmt::Display;

use async_trait::async_trait;
use eyre::Report;
use serde::Serialize;

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

pub trait Query {
    const PREFIX: Prefix;

    fn prefix(&self) -> Prefix {
        Self::PREFIX
    }
}

#[async_trait]
pub trait Respond<S> {
    type Key: Serialize;
    type Response: Serialize + Send + Sync;

    async fn respond(&self, state: &S) -> eyre::Result<(Self::Key, Self::Response)>;
}
