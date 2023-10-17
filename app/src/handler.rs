use std::sync::Arc;

use async_trait::async_trait;
use filament_chain::Transaction;
use penumbra_storage::{StateRead, StateWrite};

mod input;
mod transaction;

// TODO(xla): Document.
#[async_trait]
pub trait Handler {
    async fn validate(&self, tx: Arc<Transaction>) -> eyre::Result<()>;
    async fn check<S: StateRead>(&self, state: Arc<S>) -> eyre::Result<()>;
    async fn execute<S: StateWrite>(&self, state: &mut S) -> eyre::Result<()>;
}
