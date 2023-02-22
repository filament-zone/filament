use std::sync::Arc;

use async_trait::async_trait;
use penumbra_storage::{StateRead, StateWrite};
use pulzaar_chain::{input::Transfer, Transaction};

use crate::handler::Handler;

#[async_trait]
impl Handler for Transfer {
    async fn validate(&self, _tx: Arc<Transaction>) -> eyre::Result<()> {
        // TODO(xla): Check if asset is part of the registry.
        Ok(())
    }

    async fn check<S: StateRead>(&self, _state: Arc<S>) -> eyre::Result<()> {
        todo!()
    }

    async fn execute<S: StateWrite>(&self, _state: &mut S) -> eyre::Result<()> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use pulzaar_chain::{input, Address, Auth, ChainId, Input, Transaction, TransactionBody};
    use pulzaar_crypto::{SignBytes as _, SigningKey};
    use rand::thread_rng;

    use crate::handler::Handler as _;

    #[tokio::test]
    async fn transfer_validate() -> eyre::Result<()> {
        let signer = SigningKey::new(thread_rng());
        let transfer = input::Transfer {
            from: Address::from(signer.verification_key()),
            to: Address::from(SigningKey::new(thread_rng()).verification_key()),
            denom: "upulzaar".to_owned(),
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
        tx.validate(tx.clone()).await?;

        Ok(())
    }
}
