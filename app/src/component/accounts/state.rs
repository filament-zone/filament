// FIXME(xla): Remove.
#![allow(dead_code)]

use async_trait::async_trait;
use pulzaar_chain::Account;
use pulzaar_crypto::Address;
use pulzaar_encoding::{StateReadDecode, StateWriteEncode};

use crate::state::StateKey as _;

#[inline]
fn latest_id_key() -> String {
    "latest_account_id".to_string()
}

fn state_key_by_address(address: &Address) -> String {
    format!("accounts/{}", address.state_key())
}

#[async_trait]
pub trait AccountsRead: StateReadDecode {
    async fn account(&self, address: &Address) -> eyre::Result<Option<Account>> {
        let key = state_key_by_address(address);
        self.get_bcs::<Account>(&key).await
    }
}

impl<T: StateReadDecode + ?Sized> AccountsRead for T {}

#[async_trait]
pub trait AccountsWrite: StateWriteEncode {
    async fn create_account(&mut self, address: Address) -> eyre::Result<()> {
        let key = state_key_by_address(&address);
        if self.get_bcs::<Account>(&key).await?.is_some() {
            return Err(eyre::eyre!("accout exists already"));
        }

        let id = self.increment_id().await?;
        let key = state_key_by_address(&address);

        self.put_bcs(
            key,
            &Account::Single {
                address,
                id,
                sequence: 0,
            },
        )
    }

    async fn increment_id(&mut self) -> eyre::Result<u64> {
        let old = self
            .get_bcs::<u64>(&latest_id_key())
            .await?
            .unwrap_or_default();
        let new = old + 1;
        self.put_bcs(latest_id_key(), &new)?;

        Ok(new)
    }
}

impl<T: StateWriteEncode + ?Sized> AccountsWrite for T {}

#[cfg(test)]
mod test {
    use penumbra_storage::{StateDelta, Storage};
    use pulzaar_crypto::{Address, SigningKey};
    use rand::thread_rng;
    use tempfile::tempdir;

    use super::AccountsWrite as _;

    #[tokio::test]
    async fn create_account() -> eyre::Result<()> {
        let dir = tempdir()?;
        let path = dir.into_path();
        let storage = Storage::load(path.clone())
            .await
            .map_err(|e| eyre::eyre!(e))?;

        let mut state = StateDelta::new(storage.latest_snapshot());
        let mut state_tx = StateDelta::new(&mut state);

        let signer = SigningKey::new(thread_rng());
        let addr = Address::from(signer.verification_key());

        // First time should succeed.
        state_tx.create_account(addr.clone()).await?;

        // Subsequent creations for the same address should fail.
        assert!(state_tx.create_account(addr.clone()).await.is_err());

        state_tx.apply();

        storage.commit(state).await.unwrap();

        Ok(())
    }
}
