use async_trait::async_trait;
use filament_chain::{Address, Amount, AssetId};
use filament_encoding::{StateReadDecode, StateWriteEncode};
use num_traits::ops::checked::{CheckedAdd as _, CheckedSub as _};

use crate::component::assets::Error;

mod state_key {
    use filament_chain::{Address, AssetId};

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

#[async_trait]
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

    async fn transfer_balance(
        &mut self,
        from: &Address,
        to: &Address,
        asset_id: &AssetId,
        amount: Amount,
    ) -> eyre::Result<()> {
        let from_balance = self
            .get_balance(from, asset_id)
            .await?
            .ok_or(Error::InsufficientFunds(from.clone()))?;

        if from_balance < amount {
            eyre::bail!(Error::InsufficientFunds(from.clone()));
        }

        let to_balance = self.get_balance(to, asset_id).await?.unwrap_or(0.into());

        {
            let balance = from_balance
                .checked_sub(&amount)
                .ok_or(Error::BalanceUnderflow)?;
            self.put_balance(from, asset_id, balance)?;
        }
        {
            let balance = to_balance
                .checked_add(&amount)
                .ok_or(Error::BalanceOverflow)?;
            self.put_balance(to, asset_id, balance)?;
        }

        Ok(())
    }
}

impl<T: StateWriteEncode + ?Sized> AssetsWrite for T {}

#[cfg(test)]
mod test {
    use filament_chain::{Address, Amount, AssetId};
    use filament_crypto::SigningKey;
    use penumbra_storage::{StateDelta, TempStorage};
    use pretty_assertions::assert_eq;
    use rand::thread_rng;

    use super::{AssetsRead as _, AssetsWrite as _};
    use crate::component::assets::Error;

    #[tokio::test]
    async fn put_and_get_balance() -> eyre::Result<()> {
        let storage = TempStorage::new().await.map_err(|e| eyre::eyre!(e))?;

        let signer = SigningKey::new(thread_rng());
        let addr = Address::from(signer.verification_key());
        let id = AssetId::try_from("ufilament")?;
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

    #[tokio::test]
    async fn transfer_balance() -> eyre::Result<()> {
        let storage = TempStorage::new().await.map_err(|e| eyre::eyre!(e))?;

        let asset_id = AssetId::try_from("ugn")?;
        let from = Address::from(SigningKey::new(thread_rng()).verification_key());
        let to = Address::from(SigningKey::new(thread_rng()).verification_key());

        // Test without existing balance.
        {
            let mut state = StateDelta::new(storage.latest_snapshot());

            assert_eq!(
                state
                    .transfer_balance(&from, &to, &asset_id, 1000.into())
                    .await
                    .unwrap_err()
                    .downcast::<Error>()
                    .unwrap(),
                Error::InsufficientFunds(from.clone())
            );
        }

        // Test with balance but insufficient funds.
        {
            let mut state = StateDelta::new(storage.latest_snapshot());
            state.put_balance(&from, &asset_id, 1000.into())?;

            assert_eq!(
                state
                    .transfer_balance(&from, &to, &asset_id, 1001.into())
                    .await
                    .unwrap_err()
                    .downcast::<Error>()
                    .unwrap(),
                Error::InsufficientFunds(from.clone())
            );
        }

        // Test with existing balance and sufficient funds.
        {
            let mut state = StateDelta::new(storage.latest_snapshot());
            state.put_balance(&from, &asset_id, 1001.into())?;

            assert!(state
                .transfer_balance(&from, &to, &asset_id, 1000.into())
                .await
                .is_ok());

            storage.commit(state).await.map_err(|e| eyre::eyre!(e))?;
        }

        // Check that from account has a new balance.
        {
            let state = StateDelta::new(storage.latest_snapshot());
            let from_balance = state.get_balance(&from, &asset_id).await?.unwrap();

            assert_eq!(from_balance, 1.into());
        }

        // Check that to acocunt has updated balance.
        {
            let state = StateDelta::new(storage.latest_snapshot());
            let to_balance = state.get_balance(&to, &asset_id).await?.unwrap();

            assert_eq!(to_balance, 1000.into());
        }

        Ok(())
    }
}
