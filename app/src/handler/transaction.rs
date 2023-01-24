use std::sync::Arc;

use async_trait::async_trait;
use penumbra_storage::{State, StateTransaction};
use pulzaar_chain::Transaction;

use super::Handler;

#[async_trait]
impl Handler for Transaction {
    async fn validate(&self, _tx: Arc<Transaction>) -> eyre::Result<()> {
        todo!()
    }

    async fn check(&self, _state: Arc<State>) -> eyre::Result<()> {
        todo!()
    }

    async fn execute<'a>(&self, _state: &mut StateTransaction<'a>) -> eyre::Result<()> {
        todo!()
    }
}
