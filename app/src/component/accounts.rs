use async_trait::async_trait;
use penumbra_storage::{StateRead, StateWrite};
use pulzaar_chain::genesis::AppState;
use pulzaar_encoding::{from_bytes, to_bytes, ToBech32 as _};
use tendermint::abci::{request, response, Code};

use crate::{component::ABCIComponent, query::Prefix};

mod query;
mod state;

pub use query::Query;
pub use state::{AccountsRead, AccountsWrite};

pub struct Accounts {}

#[async_trait]
impl ABCIComponent for Accounts {
    const QUERY_PREFIX: Option<Prefix> = Some(Prefix::Accounts);

    async fn init_chain<S: StateWrite>(&self, state: &mut S, app_state: &AppState) {
        for allocation in &app_state.allocations {
            // FIXME(xla): ABCI does not allow for errors to be returned during chain
            // initialisation. Which leaves aborting the program as only alternative for now.
            state
                .create_account(allocation.address.clone())
                .await
                .unwrap();
        }
    }

    async fn query<S: StateRead>(
        &self,
        state: &S,
        query: &request::Query,
    ) -> eyre::Result<response::Query> {
        let resp = response::Query {
            height: query.height,
            codespace: Self::QUERY_PREFIX.unwrap().to_string(),
            ..response::Query::default()
        };

        let accounts_query: Query = from_bytes(&query.data)?;

        match accounts_query {
            Query::AccountByAddress(address) => Ok(match state.account(&address).await? {
                Some(account) => response::Query {
                    code: Code::Ok,
                    key: address.to_bech32()?.into(),
                    value: to_bytes(&account)?.into(),
                    ..resp
                },
                None => response::Query {
                    // TODO(tsenart): Formalize error codes ala https://github.com/cosmos/cosmos-sdk/blob/main/types/errors/errors.go
                    code: Code::from(38),
                    log: "not found".to_string(),
                    ..resp
                },
            }),
        }
    }

    async fn begin_block<S: StateWrite>(&self, _state: &mut S, _begin_block: &request::BeginBlock) {
    }

    async fn end_block<S: StateWrite>(&self, _state: &mut S, _end_block: &request::EndBlock) {}
}

#[cfg(test)]
mod test {
    use penumbra_storage::{StateDelta, Storage};
    use pulzaar_chain::{
        genesis::{Allocation, AppState},
        Address,
        Amount,
        ChainId,
        ChainParameters,
    };
    use pulzaar_crypto::SigningKey;
    use rand::{thread_rng, Rng as _};
    use tempfile::tempdir;

    use super::Accounts;
    use crate::component::{accounts::AccountsRead, ABCIComponent as _};

    #[tokio::test]
    async fn init_chain_allocations() -> eyre::Result<()> {
        let dir = tempdir()?;
        let path = dir.into_path();
        let storage = Storage::load(path.clone())
            .await
            .map_err(|e| eyre::eyre!(e))?;

        let amount = thread_rng().gen_range(0..16);
        let addresses: Vec<Address> = (0..amount)
            .map(|_| {
                let sk = SigningKey::new(thread_rng());
                Address::from(sk.verification_key())
            })
            .collect();
        let allocations = addresses
            .iter()
            .map(|address| Allocation {
                address: address.clone(),
                denom: "upulzaar".to_string(),
                amount: Amount::from(1000),
            })
            .collect();

        // Run init chain on the Accounts component with a populated set of allocations.
        {
            let mut state = StateDelta::new(storage.latest_snapshot());

            let app_state = AppState {
                allocations,
                chain_parameters: ChainParameters {
                    chain_id: ChainId::try_from("inprocess-testnet".to_string())?,
                    epoch_duration: 1024,
                },
            };

            let accounts = Accounts {};
            accounts.init_chain(&mut state, &app_state).await;

            storage.commit(state).await.map_err(|e| eyre::eyre!(e))?;
        }

        // Assert that for every address in the genesis allocation there is an account in the
        // state.
        let state = StateDelta::new(storage.latest_snapshot());

        for addr in &addresses {
            assert!(state.account(addr).await?.is_some());
        }

        Ok(())
    }
}
