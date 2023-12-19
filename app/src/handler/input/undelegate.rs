use std::sync::Arc;

use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use filament_chain::{input::Undelegate, Transaction};

use crate::handler::Handler;

#[async_trait]
impl Handler for Undelegate {
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
