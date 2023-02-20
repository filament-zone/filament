use std::sync::Arc;

use async_trait::async_trait;
use penumbra_storage::{StateRead, StateWrite};
use pulzaar_chain::{Account, Address, Auth, ChainParameters, Transaction};
use pulzaar_crypto::SignBytes;

use super::Handler;
use crate::{
    component::accounts::{AccountsRead as _, AccountsWrite},
    state::StateReadExt as _,
};

#[async_trait]
impl Handler for Transaction {
    async fn validate(&self, tx: Arc<Transaction>) -> eyre::Result<()> {
        // Ensure there is at least one input in the body.
        if self.body.inputs.is_empty() {
            return Err(eyre::eyre!("expected at least one input"));
        }

        // Verify signature matches expected given the tranaction body as message.
        let (vk, sig) = match self.auth {
            Auth::Ed25519 {
                verification_key: vk,
                signature: sig,
            } => (vk, sig),
        };

        vk.verify(&sig, &tx.body.sign_bytes()?)?;

        // TODO(xla): Execute concurretly.
        for input in self.inputs() {
            input.validate(tx.clone()).await?;
        }

        Ok(())
    }

    async fn check<S: StateRead>(&self, state: Arc<S>) -> eyre::Result<()> {
        // FIXME(xla): Loading the entire parameters every time the chain id is asserted seems
        // excessive.
        // NOTE(pm): think in general we can consider chain params to be static over long enough
        // periods to cache them
        match state.get_chain_parameters().await? {
            Some(ChainParameters { chain_id, .. }) if chain_id != self.body.chain_id => {
                return Err(eyre::eyre!(
                    "chain id missmatch: {:?} != {:?}",
                    self.body.chain_id,
                    chain_id
                ))
            },
            None => return Err(eyre::eyre!("missing chain parameters in state")),
            _ => {},
        }

        // Check for max height being within range.
        if let Some(max_height) = self.body.max_height {
            let current_height = state.get_current_height().await?;
            if max_height < current_height {
                return Err(eyre::eyre!(
                    "max_height exceeds current_height: {max_height} < {current_height}"
                ));
            }
        }

        // Check for account existence and id match.
        let address = Address::from(&self.auth);
        let sequence = match state.account(&address).await? {
            Some(Account::Single { id, .. }) if id != self.body.account_id => {
                return Err(eyre::eyre!(
                    "account id mismatch: {} != {}",
                    id,
                    self.body.account_id
                ))
            },
            None => return Err(eyre::eyre!("account doesn't exist: {address:?}")),
            Some(Account::Single { sequence, .. }) => sequence,
        };

        // Check that sequence matches.
        if sequence != self.body.sequence {
            return Err(eyre::eyre!(
                "sequence mismatch: {} != {}",
                sequence,
                self.body.sequence
            ));
        }

        // TODO(xla): Execute concurretly.
        for input in self.inputs() {
            input.check(state.clone()).await?;
        }

        Ok(())
    }

    async fn execute<S: StateWrite>(&self, state: &mut S) -> eyre::Result<()> {
        for input in self.inputs() {
            input.execute(state).await?;
        }

        let address = Address::from(&self.auth);
        state.increment_sequence(&address).await
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use penumbra_storage::{StateDelta, Storage};
    use pulzaar_chain::{
        input,
        Address,
        Auth,
        ChainId,
        ChainParameters,
        Input,
        Transaction,
        TransactionBody,
    };
    use pulzaar_crypto::{SignBytes, SigningKey};
    use rand::thread_rng;
    use tempfile::tempdir;

    use crate::{
        component::accounts::AccountsWrite as _,
        handler::Handler,
        state::StateWriteExt as _,
    };

    #[tokio::test]
    async fn validate_inputs_not_empty() -> eyre::Result<()> {
        let signer = SigningKey::new(thread_rng());
        let body = TransactionBody {
            inputs: vec![],
            chain_id: ChainId::try_from("test".to_string())?,
            max_height: None,
            account_id: 0,
            sequence: 0,
        };
        let signature = signer.sign(&body.sign_bytes()?);
        let tx = Transaction {
            auth: Auth::Ed25519 {
                verification_key: signer.verification_key(),
                signature,
            },
            body: body.clone(),
        };

        let tx = Arc::new(tx);
        assert!(tx.validate(tx.clone()).await.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn validate_signature() -> eyre::Result<()> {
        let signer = SigningKey::new(thread_rng());
        let body = TransactionBody {
            inputs: vec![Input::Delegate(input::Delegate {})],
            chain_id: ChainId::try_from("test".to_string())?,
            max_height: None,
            account_id: 0,
            sequence: 0,
        };

        // Valid signature.
        {
            let signature = signer.sign(&body.sign_bytes()?);
            let tx = Transaction {
                auth: Auth::Ed25519 {
                    verification_key: signer.verification_key(),
                    signature,
                },
                body: body.clone(),
            };
            let tx = Arc::new(tx);
            tx.validate(tx.clone()).await?;
        }

        // Invalid signature.
        {
            let signature = signer.sign(b"bogus");
            let tx = Transaction {
                auth: Auth::Ed25519 {
                    verification_key: signer.verification_key(),
                    signature,
                },
                body,
            };
            let tx = Arc::new(tx);
            assert!(tx.validate(tx.clone()).await.is_err());
        }

        Ok(())
    }

    #[tokio::test]
    async fn check_chain_id() -> eyre::Result<()> {
        let dir = tempdir()?;
        let path = dir.into_path();
        let storage = Storage::load(path.clone())
            .await
            .map_err(|e| eyre::eyre!(e))?;
        let signer = SigningKey::new(thread_rng());
        let address = Address::from(signer.verification_key());

        let chain_id = ChainId::try_from("inprocess-testnet".to_string())?;

        // Persist initial state:
        // * block height
        // * chain parameters
        // * account
        {
            let mut state = StateDelta::new(storage.latest_snapshot());
            let mut state_tx = StateDelta::new(&mut state);

            state_tx.put_chain_parameters(&ChainParameters {
                chain_id: chain_id.clone(),
                epoch_duration: 0,
            })?;
            state_tx.create_account(address).await?;

            state_tx.apply();
            storage.commit(state).await.map_err(|e| eyre::eyre!(e))?;
        }

        // Sending a transaction carrying the correct chain id should succeed.
        {
            let body = TransactionBody {
                inputs: vec![Input::Delegate(input::Delegate {})],
                chain_id,
                max_height: None,
                account_id: 0,
                sequence: 0,
            };
            let signature = signer.sign(&body.sign_bytes()?);
            let tx = Transaction {
                auth: Auth::Ed25519 {
                    verification_key: signer.verification_key(),
                    signature,
                },
                body: body.clone(),
            };

            let state = StateDelta::new(storage.latest_snapshot());
            let state = Arc::new(state);

            tx.check(state).await?;
        }

        // A transaction with the wrong chain_id should fail.
        let body = TransactionBody {
            inputs: vec![Input::Delegate(input::Delegate {})],
            chain_id: ChainId::try_from("not-your-testnet".to_string())?,
            max_height: None,
            account_id: 0,
            sequence: 0,
        };
        let signature = signer.sign(&body.sign_bytes()?);
        let tx = Transaction {
            auth: Auth::Ed25519 {
                verification_key: signer.verification_key(),
                signature,
            },
            body: body.clone(),
        };

        let state = StateDelta::new(storage.latest_snapshot());
        let state = Arc::new(state);

        assert!(tx.check(state).await.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn check_max_height() -> eyre::Result<()> {
        let dir = tempdir()?;
        let path = dir.into_path();
        let storage = Storage::load(path.clone())
            .await
            .map_err(|e| eyre::eyre!(e))?;
        let signer = SigningKey::new(thread_rng());
        let address = Address::from(signer.verification_key());

        let chain_id = ChainId::try_from("inprocess-testnet".to_string())?;

        // Persist initial state:
        // * block height
        // * chain parameters
        // * account
        {
            let mut state = StateDelta::new(storage.latest_snapshot());
            let mut state_tx = StateDelta::new(&mut state);

            state_tx.put_block_height(123)?;
            state_tx.put_chain_parameters(&ChainParameters {
                chain_id: chain_id.clone(),
                epoch_duration: 0,
            })?;
            state_tx.create_account(address).await?;

            state_tx.apply();
            storage.commit(state).await.map_err(|e| eyre::eyre!(e))?;
        }

        // Sending a transaction without a max_height should always succeed.
        {
            let body = TransactionBody {
                inputs: vec![Input::Delegate(input::Delegate {})],
                chain_id: chain_id.clone(),
                max_height: None,
                account_id: 0,
                sequence: 0,
            };
            let signature = signer.sign(&body.sign_bytes()?);
            let tx = Transaction {
                auth: Auth::Ed25519 {
                    verification_key: signer.verification_key(),
                    signature,
                },
                body: body.clone(),
            };

            let state = StateDelta::new(storage.latest_snapshot());
            let state = Arc::new(state);

            tx.check(state).await?;
        }

        // A transaction with a max_height in the future should succeed.
        {
            let body = TransactionBody {
                inputs: vec![Input::Delegate(input::Delegate {})],
                chain_id: chain_id.clone(),
                max_height: Some(124),
                account_id: 0,
                sequence: 0,
            };
            let signature = signer.sign(&body.sign_bytes()?);
            let tx = Transaction {
                auth: Auth::Ed25519 {
                    verification_key: signer.verification_key(),
                    signature,
                },
                body: body.clone(),
            };

            let state = StateDelta::new(storage.latest_snapshot());
            let state = Arc::new(state);

            tx.check(state).await?;
        }

        // A transaction with a height smaller than the current height should fail.
        let body = TransactionBody {
            inputs: vec![Input::Delegate(input::Delegate {})],
            chain_id: chain_id.clone(),
            max_height: Some(122),
            account_id: 0,
            sequence: 0,
        };
        let signature = signer.sign(&body.sign_bytes()?);
        let tx = Transaction {
            auth: Auth::Ed25519 {
                verification_key: signer.verification_key(),
                signature,
            },
            body: body.clone(),
        };

        let state = StateDelta::new(storage.latest_snapshot());
        let state = Arc::new(state);

        assert!(tx.check(state).await.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn check_account_id_single() -> eyre::Result<()> {
        let dir = tempdir()?;
        let path = dir.into_path();
        let storage = Storage::load(path.clone())
            .await
            .map_err(|e| eyre::eyre!(e))?;
        let signer = SigningKey::new(thread_rng());
        let address = Address::from(signer.verification_key());

        let chain_id = ChainId::try_from("inprocess-testnet".to_string())?;

        // Persist initial state:
        // * block height
        // * chain parameters
        // * account
        {
            let mut state = StateDelta::new(storage.latest_snapshot());
            let mut state_tx = StateDelta::new(&mut state);

            state_tx.put_block_height(1)?;
            state_tx.put_chain_parameters(&ChainParameters {
                chain_id: chain_id.clone(),
                epoch_duration: 0,
            })?;
            // Create a additional accounts to avoid that assertions are on the default of the
            // type.
            state_tx
                .create_account(Address::from(
                    SigningKey::new(thread_rng()).verification_key(),
                ))
                .await?;
            state_tx
                .create_account(Address::from(
                    SigningKey::new(thread_rng()).verification_key(),
                ))
                .await?;
            state_tx
                .create_account(Address::from(
                    SigningKey::new(thread_rng()).verification_key(),
                ))
                .await?;

            state_tx.create_account(address).await?;

            state_tx.apply();
            storage.commit(state).await.map_err(|e| eyre::eyre!(e))?;
        }

        // Sending a transaction with the correct account id should succeed.
        {
            let body = TransactionBody {
                inputs: vec![Input::Delegate(input::Delegate {})],
                chain_id: chain_id.clone(),
                max_height: None,
                account_id: 3,
                sequence: 0,
            };
            let signature = signer.sign(&body.sign_bytes()?);
            let tx = Transaction {
                auth: Auth::Ed25519 {
                    verification_key: signer.verification_key(),
                    signature,
                },
                body: body.clone(),
            };

            let state = StateDelta::new(storage.latest_snapshot());
            let state = Arc::new(state);

            tx.check(state).await?;
        }

        // A transaction from an account that has not been created should fail.
        let body = TransactionBody {
            inputs: vec![Input::Delegate(input::Delegate {})],
            chain_id: chain_id.clone(),
            max_height: None,
            account_id: 4,
            sequence: 0,
        };
        let signer = SigningKey::new(thread_rng());
        let signature = signer.sign(&body.sign_bytes()?);
        let tx = Transaction {
            auth: Auth::Ed25519 {
                verification_key: signer.verification_key(),
                signature,
            },
            body: body.clone(),
        };

        let state = StateDelta::new(storage.latest_snapshot());
        let state = Arc::new(state);

        assert!(tx.check(state).await.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn check_account_sequence() -> eyre::Result<()> {
        let dir = tempdir()?;
        let path = dir.into_path();
        let storage = Storage::load(path.clone())
            .await
            .map_err(|e| eyre::eyre!(e))?;
        let signer = SigningKey::new(thread_rng());
        let address = Address::from(signer.verification_key());

        let chain_id = ChainId::try_from("inprocess-testnet".to_string())?;

        // Persist initial state:
        // * block height
        // * chain parameters
        // * account
        {
            let mut state = StateDelta::new(storage.latest_snapshot());
            let mut state_tx = StateDelta::new(&mut state);

            state_tx.put_block_height(1)?;
            state_tx.put_chain_parameters(&ChainParameters {
                chain_id: chain_id.clone(),
                epoch_duration: 0,
            })?;

            state_tx.create_account(address.clone()).await?;

            state_tx.apply();
            storage.commit(state).await.map_err(|e| eyre::eyre!(e))?;
        }

        // Sending a transaction with the correct account id should succeed.
        {
            let body = TransactionBody {
                inputs: vec![Input::Delegate(input::Delegate {})],
                chain_id: chain_id.clone(),
                max_height: None,
                account_id: 0,
                sequence: 0,
            };
            let signature = signer.sign(&body.sign_bytes()?);
            let tx = Transaction {
                auth: Auth::Ed25519 {
                    verification_key: signer.verification_key(),
                    signature,
                },
                body: body.clone(),
            };

            let state = StateDelta::new(storage.latest_snapshot());
            let state = Arc::new(state);

            tx.check(state).await?;

            // Increment sequence to assert that it can't be reused.
            let mut state = StateDelta::new(storage.latest_snapshot());
            let mut state_tx = StateDelta::new(&mut state);

            tx.execute(&mut state_tx).await?;

            state_tx.apply();
            storage.commit(state).await.map_err(|e| eyre::eyre!(e))?;
        }

        // A transaction with an old sequence should fail.
        let body = TransactionBody {
            inputs: vec![Input::Delegate(input::Delegate {})],
            chain_id: chain_id.clone(),
            max_height: None,
            account_id: 0,
            sequence: 0,
        };
        let signer = SigningKey::new(thread_rng());
        let signature = signer.sign(&body.sign_bytes()?);
        let tx = Transaction {
            auth: Auth::Ed25519 {
                verification_key: signer.verification_key(),
                signature,
            },
            body: body.clone(),
        };

        let state = StateDelta::new(storage.latest_snapshot());
        let state = Arc::new(state);

        assert!(tx.check(state).await.is_err());

        Ok(())
    }
}
