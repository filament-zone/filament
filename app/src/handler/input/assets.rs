use std::sync::Arc;

use async_trait::async_trait;
use penumbra_storage::{StateRead, StateWrite};
use pulzaar_chain::{input::Transfer, Transaction};

use crate::handler::Handler;

#[async_trait]
impl Handler for Transfer {
    async fn validate(&self, _tx: Arc<Transaction>) -> eyre::Result<()> {
        todo!()
    }

    async fn check<S: StateRead>(&self, _state: Arc<S>) -> eyre::Result<()> {
        todo!()
    }

    async fn execute<S: StateWrite>(&self, _state: &mut S) -> eyre::Result<()> {
        todo!()
    }
}
