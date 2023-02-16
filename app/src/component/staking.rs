use async_trait::async_trait;
use penumbra_storage::StateWrite;
use pulzaar_chain::genesis::AppState;
use tendermint::abci::request::{BeginBlock, EndBlock};

use super::ABCIComponent;

#[derive(Default)]
pub struct Staking {}

#[async_trait]
impl<S> ABCIComponent<S> for Staking
where
    S: StateWrite,
{
    async fn init_chain(&self, _state: &mut S, _app_state: &AppState) {
        todo!()
    }

    async fn begin_block(&self, _state: &mut S, _begin_blocke: &BeginBlock) {
        todo!()
    }

    async fn end_block(&self, _state: &mut S, _end_block: &EndBlock) {
        todo!()
    }
}
