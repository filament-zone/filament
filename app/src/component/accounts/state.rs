use async_trait::async_trait;
use pulzaar_chain::{Account, Address};
use pulzaar_encoding::{StateReadDecode, StateWriteEncode};

mod state_key {
    use pulzaar_chain::Address;

    use crate::state_key::StateKey as _;

    #[inline]
    pub fn next_account_id() -> String {
        "latest_account_id".to_string()
    }

    pub fn by_address(address: &Address) -> String {
        format!("accounts/{}", address.state_key())
    }
}

#[async_trait]
pub trait AccountsRead: StateReadDecode {
    async fn account(&self, address: &Address) -> eyre::Result<Option<Account>> {
        let key = state_key::by_address(address);
        self.get_bcs::<Account>(&key).await
    }
}

impl<T: StateReadDecode + ?Sized> AccountsRead for T {}

#[async_trait]
pub trait AccountsWrite: StateWriteEncode {
    async fn increment_id(&mut self) -> eyre::Result<u64> {
        let old = self
            .get_bcs::<u64>(&state_key::next_account_id())
            .await?
            .unwrap_or_default();
        let new = old + 1;
        self.put_bcs(state_key::next_account_id(), &new)?;

        Ok(old)
    }

    async fn create_account(&mut self, address: Address) -> eyre::Result<()> {
        let key = state_key::by_address(&address);
        if self.get_bcs::<Account>(&key).await?.is_some() {
            return Err(eyre::eyre!("accout exists already"));
        }

        let id = self.increment_id().await?;
        let key = state_key::by_address(&address);

        self.put_bcs(
            key,
            &Account::Single {
                address,
                id,
                sequence: 0,
            },
        )
    }

    async fn increment_sequence(&mut self, address: &Address) -> eyre::Result<()> {
        let key = state_key::by_address(address);
        let account = match self.get_bcs::<Account>(&key).await? {
            None => return Err(eyre::eyre!("account doesn't exist: {address:?}")),
            Some(account) => account,
        };

        match account {
            Account::Single { id, sequence, .. } => self.put_bcs(
                key,
                &Account::Single {
                    address: address.clone(),
                    id,
                    sequence: sequence + 1,
                },
            ),
        }
    }
}

impl<T: StateWriteEncode + ?Sized> AccountsWrite for T {}

#[cfg(test)]
mod test {
    use penumbra_storage::{StateDelta, Storage};
    use pretty_assertions::assert_eq;
    use pulzaar_chain::{Account, Address};
    use pulzaar_crypto::SigningKey;
    use rand::thread_rng;
    use tempfile::tempdir;

    use super::AccountsWrite as _;
    use crate::component::accounts::AccountsRead;

    // TODO(xla): Remove once Account has more than one variant.
    #[allow(irrefutable_let_patterns)]
    #[tokio::test]
    async fn create_account() -> eyre::Result<()> {
        let dir = tempdir()?;
        let path = dir.into_path();
        let storage = Storage::load(path.clone())
            .await
            .map_err(|e| eyre::eyre!(e))?;

        let signer = SigningKey::new(thread_rng());
        let addr = Address::from(signer.verification_key());

        {
            let mut state = StateDelta::new(storage.latest_snapshot());
            let mut state_tx = StateDelta::new(&mut state);

            // First time should succeed.
            state_tx.create_account(addr.clone()).await?;

            // Subsequent creations for the same address should fail.
            assert!(state_tx.create_account(addr.clone()).await.is_err());

            state_tx.apply();

            storage.commit(state).await.unwrap();
        }

        // Account should be present in the state.
        let state = StateDelta::new(storage.latest_snapshot());
        let account = state.account(&addr).await?.unwrap();

        if let Account::Single {
            address,
            id,
            sequence,
        } = account
        {
            assert_eq!(address, addr);
            assert_eq!(id, 0);
            assert_eq!(sequence, 0);
        } else {
            panic!("expected single account in state");
        }

        Ok(())
    }
}
