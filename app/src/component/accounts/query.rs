use async_trait::async_trait;
use pulzaar_chain::{Account, Address};
use pulzaar_encoding::StateReadDecode;
use serde::{Deserialize, Serialize};

use crate::{component::accounts::AccountsRead as _, query};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum Query {
    AccountByAddress(Address),
}

#[async_trait]
impl<S> query::Query<S> for Query
where
    S: StateReadDecode,
{
    type Key = Vec<u8>;
    type Response = Response;

    async fn respond(&self, state: &S) -> eyre::Result<(Vec<u8>, Response)> {
        match self {
            Self::AccountByAddress(address) => {
                let account = state
                    .account(address)
                    .await?
                    .ok_or(eyre::eyre!("account not found"))?;

                Ok((
                    pulzaar_encoding::to_bytes(&address)?.to_vec(),
                    Response::Account(account),
                ))
            },
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Response {
    Account(Account),
}
