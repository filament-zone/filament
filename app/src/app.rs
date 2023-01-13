use std::sync::Arc;

use penumbra_storage::State;
use pulzaar_chain::genesis::AppState;

use crate::Component;

/// The Pulzaar ABCI application modeled as stack of [`Component`]s.
pub struct App {
    components: Vec<Box<dyn Component>>,
    state: Arc<State>,
}

impl App {
    pub fn new(state: State) -> Self {
        Self {
            components: vec![],
            state: Arc::new(state),
        }
    }

    pub async fn init_chain(&mut self, app_state: &AppState) {
        let state = Arc::get_mut(&mut self.state).expect("failed to acquire state reference");
        let mut state_tx = state.begin_transaction();

        for component in &self.components {
            component.init_chain(&mut state_tx, app_state).await;
        }

        state_tx.apply();
    }
}
