use async_trait::async_trait;
use pulzaar_chain::{Address, Amount, AssetId};
use pulzaar_encoding::{StateReadDecode, StateWriteEncode};

mod state_key {
    use pulzaar_chain::{Address, AssetId};

    use crate::state_key::StateKey as _;

    pub fn balance(address: &Address, asset_id: &AssetId) -> String {
        format!("balances/{}/{}", address.state_key(), asset_id.state_key())
    }
}

#[async_trait]
pub trait AssetsRead: StateReadDecode {
    async fn get_balance(
        &self,
        address: &Address,
        asset_id: &AssetId,
    ) -> eyre::Result<Option<Amount>> {
        self.get_bcs::<Amount>(&state_key::balance(address, asset_id))
            .await
    }
}

impl<T: StateReadDecode + ?Sized> AssetsRead for T {}

pub trait AssetsWrite: StateWriteEncode {
    fn put_balance(
        &mut self,
        address: &Address,
        asset_id: &AssetId,
        amount: Amount,
    ) -> eyre::Result<()> {
        let key = state_key::balance(address, asset_id);
        self.put_bcs(key, &amount)
    }
}

impl<T: StateWriteEncode + ?Sized> AssetsWrite for T {}

#[cfg(test)]
mod test {
    use penumbra_storage::{StateDelta, Storage};
    use pretty_assertions::assert_eq;
    use pulzaar_chain::{Address, Amount, AssetId};
    use pulzaar_crypto::SigningKey;
    use rand::thread_rng;
    use tempfile::tempdir;

    use super::{AssetsRead as _, AssetsWrite as _};

    #[tokio::test]
    async fn put_and_get_balance() -> eyre::Result<()> {
        let dir = tempdir()?;
        let path = dir.into_path();
        let storage = Storage::load(path.clone())
            .await
            .map_err(|e| eyre::eyre!(e))?;

        let signer = SigningKey::new(thread_rng());
        let addr = Address::from(signer.verification_key());
        let id = AssetId::try_from("upulzaar")?;
        let amount: u128 = rand::random();
        let amount = Amount::from(amount);

        // Write balance to state.
        {
            let mut state = StateDelta::new(storage.latest_snapshot());
            state.put_balance(&addr, &id, amount)?;

            storage.commit(state).await.map_err(|e| eyre::eyre!(e))?;
        }

        // Assert balance is correctly stored.
        let state = StateDelta::new(storage.latest_snapshot());

        let balance = state.get_balance(&addr, &id).await?;

        assert_eq!(balance, Some(amount));

        Ok(())
    }
}
