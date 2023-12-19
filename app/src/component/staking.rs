use async_trait::async_trait;
use cnidarium::StateWrite;
use filament_chain::genesis::AppState;
use tendermint::abci::request;

use crate::component::ABCIComponent;

#[derive(Default)]
pub struct Staking {}

#[async_trait]
impl ABCIComponent for Staking {
    async fn init_chain<S: StateWrite>(&self, _state: &mut S, _app_state: &AppState) {}

    async fn begin_block<S: StateWrite>(&self, _state: &mut S, _begin_block: &request::BeginBlock) {
    }

    async fn end_block<S: StateWrite>(&self, _state: &mut S, _end_block: &request::EndBlock) {}
}
