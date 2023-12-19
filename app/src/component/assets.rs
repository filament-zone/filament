use async_trait::async_trait;
use cnidarium::StateWrite;
use filament_chain::{genesis::AppState, REGISTRY};
use tendermint::abci::request;

use crate::component::ABCIComponent;

mod error;
mod state;

pub use error::Error;
pub use state::{AssetsRead, AssetsWrite};

pub struct Assets {}

#[async_trait]
impl ABCIComponent for Assets {
    async fn init_chain<S: StateWrite>(&self, state: &mut S, app_state: &AppState) {
        // Store account allocations.
        for allocation in &app_state.allocations {
            let asset = REGISTRY
                .by_base_denom(&allocation.denom)
                .expect("asset not found for denom");

            // FIXME(xla): ABCI does not allow for errors to be returned during chain
            // initialisation. Which leaves aborting the program as only alternative for now.
            state
                .put_balance(&allocation.address, &asset.id, allocation.amount)
                .unwrap();
        }
    }

    async fn begin_block<S: StateWrite>(&self, _state: &mut S, _begin_block: &request::BeginBlock) {
    }

    async fn end_block<S: StateWrite>(&self, _state: &mut S, _end_block: &request::EndBlock) {}
}

#[cfg(test)]
mod test {
    use cnidarium::{StateDelta, Storage};
    use filament_chain::{
        genesis::{Allocation, AppState},
        Address,
        Amount,
        AssetId,
        ChainId,
        ChainParameters,
    };
    use filament_crypto::SigningKey;
    use pretty_assertions::assert_eq;
    use rand::{thread_rng, Rng as _};
    use tempfile::tempdir;

    use super::Assets;
    use crate::component::{assets::AssetsRead as _, ABCIComponent as _};

    #[tokio::test]
    async fn init_chain_allocations() -> eyre::Result<()> {
        let dir = tempdir()?;
        let path = dir.into_path();
        let storage = Storage::load(path.clone(), vec![])
            .await
            .map_err(|e| eyre::eyre!(e))?;

        let amount = thread_rng().gen_range(0..16);
        let allocations = (0..amount)
            .map(|_| {
                let sk = SigningKey::new(thread_rng());
                Address::from(sk.verification_key())
            })
            .map(|address| Allocation {
                address,
                denom: "ugm".to_string(),
                amount: Amount::from(1000),
            })
            .collect::<Vec<_>>();

        // Run init chain on the Assets component with a populated set of allocations.
        {
            let mut state = StateDelta::new(storage.latest_snapshot());

            let app_state = AppState {
                allocations: allocations.clone(),
                chain_parameters: ChainParameters {
                    chain_id: ChainId::try_from("inprocess-testnet".to_string())?,
                    epoch_duration: 1024,
                },
            };

            let assets = Assets {};
            assets.init_chain(&mut state, &app_state).await;

            storage.commit(state).await.map_err(|e| eyre::eyre!(e))?;
        }

        // Assert that for every allocation there is a corresponding balance in the state.
        let state = StateDelta::new(storage.latest_snapshot());

        for allocation in &allocations {
            let id = AssetId::try_from(allocation.denom.as_ref())?;
            let balance = state.get_balance(&allocation.address, &id).await?;
            assert_eq!(balance, Some(allocation.amount));
        }

        Ok(())
    }
}
