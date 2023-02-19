use std::sync::Arc;

use async_trait::async_trait;
use penumbra_storage::{StateRead, StateWrite};
use pulzaar_chain::{Auth, Transaction};
use pulzaar_crypto::SignBytes;

use super::Handler;

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
        // TODO(xla): Assert that chain_id matches.
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

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use pulzaar_chain::{input, Auth, Input, Transaction, TransactionBody};
    use pulzaar_crypto::{SignBytes, SigningKey};
    use rand::thread_rng;

    use crate::handler::Handler;

    #[tokio::test]
    async fn validate_inputs_not_empty() -> eyre::Result<()> {
        let signer = SigningKey::new(thread_rng());
        let body = TransactionBody {
            inputs: vec![],
            chain_id: "test".to_string(),
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
            chain_id: "test".to_string(),
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
}
