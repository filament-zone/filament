use std::sync::Arc;

use async_trait::async_trait;
use penumbra_storage::{StateRead, StateWrite};
use pulzaar_chain::Transaction;

use super::Handler;

#[async_trait]
impl Handler for Transaction {
    async fn validate(&self, tx: Arc<Transaction>) -> eyre::Result<()> {
        // TODO(xla): Execute concurretly.
        for input in self.inputs() {
            input.validate(tx.clone()).await?;
        }

        Ok(())
    }

    async fn check<S: StateRead>(&self, state: Arc<S>) -> eyre::Result<()> {
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
