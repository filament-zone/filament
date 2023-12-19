use std::sync::Arc;

use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use filament_chain::{input::Transfer, Address, Transaction, REGISTRY};

use crate::{
    component::{
        accounts::{AccountsRead, AccountsWrite},
        assets::{AssetsRead as _, AssetsWrite as _, Error},
    },
    handler::Handler,
};

#[async_trait]
impl Handler for Transfer {
    async fn validate(&self, tx: Arc<Transaction>) -> eyre::Result<()> {
        // FIXME(xla): This is a stop-gap until there is a proper notion of account authentication.
        if Address::from(&tx.auth) != self.from {
            eyre::bail!(Error::SenderUnauthorized);
        }

        // Check if asset is part of the registry.
        REGISTRY
            .by_base_denom(&self.denom.base)
            .ok_or(Error::AssetNotSupported(self.denom.base.clone()))?;

        Ok(())
    }

    async fn check<S: StateRead>(&self, state: Arc<S>) -> eyre::Result<()> {
        let asset = REGISTRY
            .by_base_denom(&self.denom.base)
            .ok_or(Error::AssetNotSupported(self.denom.base.clone()))?;
        let balance = state
            .get_balance(&self.from, &asset.id)
            .await?
            .ok_or(Error::InsufficientFunds(self.from.clone()))?;

        // Check if from account has sufficient funds.
        if balance < self.amount {
            eyre::bail!(Error::InsufficientFunds(self.from.clone()));
        }

        Ok(())
    }

    async fn execute<S: StateWrite>(&self, state: &mut S) -> eyre::Result<()> {
        let asset = REGISTRY
            .by_base_denom(&self.denom.base)
            .ok_or(Error::AssetNotSupported(self.denom.base.clone()))?;

        // Create recipient's account if it doesn't exist.
        if state.account(&self.to).await?.is_none() {
            state.create_account(self.to.clone()).await?;
        }

        state
            .transfer_balance(&self.from, &self.to, &asset.id, self.amount)
            .await
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use cnidarium::{StateDelta, TempStorage};
    use filament_chain::{
        input,
        Address,
        Amount,
        Asset,
        AssetId,
        Auth,
        ChainId,
        Denom,
        Input,
        Transaction,
        TransactionBody,
        REGISTRY,
    };
    use filament_crypto::{SignBytes as _, SigningKey};
    use pretty_assertions::assert_eq;
    use rand::thread_rng;

    use super::Error;
    use crate::{
        component::assets::{AssetsRead, AssetsWrite as _},
        handler::Handler as _,
    };

    #[tokio::test]
    async fn validate() -> eyre::Result<()> {
        let signer = SigningKey::new(thread_rng());

        let asset = REGISTRY.by_base_denom("ugm").unwrap();
        // Test with unsuported asset.
        let transfer = input::Transfer {
            from: Address::from(signer.verification_key()),
            to: Address::from(SigningKey::new(thread_rng()).verification_key()),
            denom: asset.denom.clone(),
            amount: rand::random::<u128>().into(),
        };

        let body = TransactionBody {
            inputs: vec![Input::Transfer(transfer)],
            chain_id: ChainId::try_from("inprocess-testnet".to_string())?,
            max_height: None,
            account_id: 0,
            sequence: 0,
        };
        let tx = Transaction {
            auth: Auth::Ed25519 {
                verification_key: signer.verification_key(),
                signature: signer.sign(&body.sign_bytes()?),
            },
            body,
        };

        let tx = Arc::new(tx);

        tx.validate(tx.clone()).await
    }

    #[tokio::test]
    async fn validate_signer_not_sender() -> eyre::Result<()> {
        let signer = SigningKey::new(thread_rng());

        let asset = REGISTRY.by_base_denom("ugm").unwrap();
        // Test with unsuported asset.
        let transfer = input::Transfer {
            from: Address::from(signer.verification_key()),
            to: Address::from(SigningKey::new(thread_rng()).verification_key()),
            denom: asset.denom.clone(),
            amount: rand::random::<u128>().into(),
        };
        let body = TransactionBody {
            inputs: vec![Input::Transfer(transfer)],
            chain_id: ChainId::try_from("inprocess-testnet".to_string())?,
            max_height: None,
            account_id: 0,
            sequence: 0,
        };
        let tx = {
            let signer = SigningKey::new(thread_rng());
            Transaction {
                auth: Auth::Ed25519 {
                    verification_key: signer.verification_key(),
                    signature: signer.sign(&body.sign_bytes()?),
                },
                body,
            }
        };

        let tx = Arc::new(tx);

        assert_eq!(
            tx.validate(tx.clone())
                .await
                .unwrap_err()
                .downcast::<Error>()
                .unwrap(),
            Error::SenderUnauthorized
        );

        Ok(())
    }

    #[tokio::test]
    async fn validate_unsupported_asset() -> eyre::Result<()> {
        let signer = SigningKey::new(thread_rng());

        let asset_id = AssetId::try_from("ugn")?;
        let asset = Asset {
            id: asset_id.clone(),
            denom: Denom {
                id: asset_id,
                base: "ugn".to_owned(),
                units: vec![],
            },
        };

        let transfer = input::Transfer {
            from: Address::from(signer.verification_key()),
            to: Address::from(SigningKey::new(thread_rng()).verification_key()),
            denom: asset.denom.clone(),
            amount: rand::random::<u128>().into(),
        };

        let body = TransactionBody {
            inputs: vec![Input::Transfer(transfer)],
            chain_id: ChainId::try_from("inprocess-testnet".to_string())?,
            max_height: None,
            account_id: 0,
            sequence: 0,
        };
        let tx = Transaction {
            auth: Auth::Ed25519 {
                verification_key: signer.verification_key(),
                signature: signer.sign(&body.sign_bytes()?),
            },
            body,
        };

        let tx = Arc::new(tx);

        assert_eq!(
            tx.validate(tx.clone())
                .await
                .unwrap_err()
                .downcast::<Error>()
                .unwrap(),
            Error::AssetNotSupported(asset.denom.base)
        );

        Ok(())
    }

    #[tokio::test]
    async fn check() -> eyre::Result<()> {
        let storage = TempStorage::new().await.map_err(|e| eyre::eyre!(e))?;

        let signer = SigningKey::new(thread_rng());
        let asset = REGISTRY.by_base_denom("ugm").unwrap();
        let from = Address::from(signer.verification_key());
        let to = Address::from(SigningKey::new(thread_rng()).verification_key());
        let initial_funds = rand::random::<u128>() + 1001;

        // Fund sender's account.
        {
            let mut state = StateDelta::new(storage.latest_snapshot());
            state.put_balance(&from, &asset.id, Amount::from(initial_funds))?;

            storage.commit(state).await.map_err(|e| eyre::eyre!(e))?;
        }

        let transfer = input::Transfer {
            from: from.clone(),
            to: to.clone(),
            denom: asset.denom.clone(),
            amount: 1000u128.into(),
        };

        let state = StateDelta::new(storage.latest_snapshot());
        let state = Arc::new(state);
        transfer.check(state).await
    }

    #[tokio::test]
    async fn check_insufficient_funds() -> eyre::Result<()> {
        let storage = TempStorage::new().await.map_err(|e| eyre::eyre!(e))?;

        let asset = REGISTRY.by_base_denom("ugm").unwrap();
        let from = Address::from(SigningKey::new(thread_rng()).verification_key());

        let transfer = input::Transfer {
            from: from.clone(),
            to: Address::from(SigningKey::new(thread_rng()).verification_key()),
            denom: asset.denom.clone(),
            amount: 1000u128.into(),
        };

        let state = StateDelta::new(storage.latest_snapshot());
        let state = Arc::new(state);

        assert_eq!(
            transfer
                .check(state)
                .await
                .unwrap_err()
                .downcast::<Error>()
                .unwrap(),
            Error::InsufficientFunds(from)
        );

        Ok(())
    }

    #[tokio::test]
    async fn execute() -> eyre::Result<()> {
        let storage = TempStorage::new().await.map_err(|e| eyre::eyre!(e))?;

        let asset = REGISTRY.by_base_denom("ugm").unwrap();
        let from = Address::from(SigningKey::new(thread_rng()).verification_key());
        let to = Address::from(SigningKey::new(thread_rng()).verification_key());
        let amount = Amount::from(1000);
        let transfer = input::Transfer {
            from: from.clone(),
            to: to.clone(),
            denom: asset.denom.clone(),
            amount,
        };

        // Fund sender account.
        {
            let mut state = StateDelta::new(storage.latest_snapshot());
            state.put_balance(&from, &asset.id, amount + amount)?;
            storage.commit(state).await.map_err(|e| eyre::eyre!(e))?;
        }

        // Execute transfer.
        {
            let mut state = StateDelta::new(storage.latest_snapshot());

            assert!(transfer.execute(&mut state).await.is_ok());
            storage.commit(state).await.map_err(|e| eyre::eyre!(e))?;
        }

        // Confirm new balances.
        {
            let state = StateDelta::new(storage.latest_snapshot());
            assert_eq!(state.get_balance(&from, &asset.id).await?.unwrap(), amount);
            assert_eq!(state.get_balance(&to, &asset.id).await?.unwrap(), amount);
        }

        Ok(())
    }
}
