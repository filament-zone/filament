use std::sync::Arc;

use async_trait::async_trait;
use cnidarium::{StateRead, StateWrite};
use filament_chain::{Input, Transaction};

use crate::handler::Handler;

mod delegate;
mod transfer;
mod undelegate;

#[async_trait]
impl Handler for Input {
    async fn validate(&self, tx: Arc<Transaction>) -> eyre::Result<()> {
        match self {
            Self::Delegate(input) => input.validate(tx),
            Self::Undelegate(input) => input.validate(tx),

            Self::Transfer(input) => input.validate(tx),
        }
        .await
    }

    async fn check<S: StateRead>(&self, state: Arc<S>) -> eyre::Result<()> {
        match self {
            Self::Delegate(input) => input.check(state),
            Self::Undelegate(input) => input.check(state),

            Self::Transfer(input) => input.check(state),
        }
        .await
    }

    async fn execute<S: StateWrite>(&self, state: &mut S) -> eyre::Result<()> {
        match self {
            Self::Delegate(input) => input.execute(state),
            Self::Undelegate(input) => input.execute(state),

            Self::Transfer(input) => input.execute(state),
        }
        .await
    }
}
