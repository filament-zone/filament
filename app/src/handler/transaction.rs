use std::sync::Arc;

use async_trait::async_trait;
use penumbra_storage::{StateRead, StateWrite};
use pulzaar_chain::{Auth, ChainParameters, Transaction};
use pulzaar_crypto::SignBytes;

use super::Handler;
use crate::state::StateReadExt as _;

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

        // TODO(xla): Asseert max_height is not exceeded.
        // TODO(xla): Assert account id matches.
        // TODO(xla): Assert sequence number matches.
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

        // TODO(xla): Update sequence number on accounts.

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use penumbra_storage::{StateDelta, Storage};
    use pulzaar_chain::{
        input,
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

    use crate::{handler::Handler, state::StateWriteExt as _};

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

        let chain_id = ChainId::try_from("inprocess-testnet".to_string())?;

        // Persist initial chain parameters including the chain id.
        {
            let mut state = StateDelta::new(storage.latest_snapshot());
            let mut state_tx = StateDelta::new(&mut state);

            state_tx.put_chain_parameters(&ChainParameters {
                chain_id: chain_id.clone(),
                epoch_duration: 0,
            })?;

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
}
