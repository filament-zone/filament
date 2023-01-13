use std::sync::Arc;

use async_trait::async_trait;
use penumbra_storage::{State, StateTransaction};
use pulzaar_chain::Transaction;

mod transaction;

#[async_trait]
pub trait Handler {
    async fn validate(&self, tx: Arc<Transaction>) -> eyre::Result<()>;
    async fn check(&self, state: Arc<State>) -> eyre::Result<()>;
    async fn execute<'a>(&self, state: &mut StateTransaction<'a>) -> eyre::Result<()>;
}
