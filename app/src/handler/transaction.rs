use std::sync::Arc;

use async_trait::async_trait;
use penumbra_storage::{State, StateTransaction};
use pulzaar_chain::Transaction;

use super::Handler;

#[async_trait]
impl Handler for Transaction {
    async fn validate(&self, tx: Arc<Transaction>) -> eyre::Result<()> {
        todo!()
    }

    async fn check(&self, state: Arc<State>) -> eyre::Result<()> {
        todo!()
    }

    async fn execute<'a>(&self, state: &mut StateTransaction<'a>) -> eyre::Result<()> {
        todo!()
    }
}
