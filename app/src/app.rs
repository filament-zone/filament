use std::sync::Arc;

use filament_chain::{genesis::AppState, Transaction};
use filament_encoding as encoding;
use penumbra_storage::{ArcStateDeltaExt as _, Snapshot, StateDelta, Storage};
use tendermint::{
    abci::{Code, Event},
    consensus::Params,
    v0_34::abci::{request, response},
    validator::Update,
};
use tracing::instrument;

use crate::{
    component::{self, ABCIComponent as _, Accounts, Assets, Component, Staking},
    handler::Handler as _,
    query::{Prefix, Respond as _},
    state::StateWriteExt as _,
    AppHash,
};

/// The filament ABCI application modeled as stack of [`Component`]s.
pub struct App {
    components: Vec<Component>,
    state: Arc<StateDelta<Snapshot>>,
}

impl App {
    pub fn new(snapshot: Snapshot) -> Self {
        Self {
            components: vec![
                Component::Accounts(Accounts {}),
                Component::Assets(Assets {}),
                Component::Staking(Staking {}),
            ],
            state: Arc::new(StateDelta::new(snapshot)),
        }
    }

    #[instrument(skip(self, app_state))]
    pub async fn init_chain(&mut self, app_state: &AppState) {
        let mut state_tx = self
            .state
            .try_begin_transaction()
            .expect("state Arc should not be referenced elsewhere");

        state_tx
            .put_chain_parameters(&app_state.chain_parameters)
            .unwrap();

        for component in &self.components {
            component.init_chain(&mut state_tx, app_state).await;
        }

        state_tx.apply();
    }

    /// * Query for data from the application at current or past height.
    /// * Optionally return Merkle proof.
    /// * Merkle proof includes self-describing type field to support many types of Merkle trees and
    ///   encoding formats.
    ///
    /// <https://github.com/tendermint/tendermint/blob/main/spec/abci/abci.md#query-1>
    #[allow(unreachable_patterns)]
    #[instrument(skip(self))]
    pub async fn query(&self, query: &request::Query) -> eyre::Result<response::Query> {
        let prefix = Prefix::try_from(query.path.as_str())?;

        let q = match prefix {
            Prefix::Accounts => {
                filament_encoding::from_bytes::<component::accounts::Query>(&query.data)?
            },
            _ => eyre::bail!("component for {} not found", query.path),
        };

        let res = response::Query {
            height: query.height,
            codespace: prefix.to_string(),
            ..response::Query::default()
        };

        let res = match q.respond(&self.state).await {
            Ok((key, data)) => response::Query {
                code: Code::Ok,
                key: filament_encoding::to_bytes(&key)?.into(),
                value: filament_encoding::to_bytes(&data)?.into(),
                ..res
            },
            Err(err) => response::Query {
                // TODO(tsenart): Formalize error codes ala https://github.com/cosmos/cosmos-sdk/blob/main/types/errors/errors.go
                code: Code::from(38),
                log: err.to_string(),
                ..res
            },
        };

        Ok(res)
    }

    #[instrument(skip(self, begin_block))]
    pub async fn begin_block(&mut self, begin_block: &request::BeginBlock) -> Vec<Event> {
        let mut state_tx = self
            .state
            .try_begin_transaction()
            .expect("state Arc should not be referenced elsewhere");

        // store the block height
        let _ = state_tx.put_block_height(begin_block.header.height.into());
        // store the block time
        let _ = state_tx.put_block_timestamp(begin_block.header.time);

        for component in &self.components {
            component.begin_block(&mut state_tx, begin_block).await;
        }

        state_tx.apply().1
    }

    #[instrument(skip(self, tx))]
    pub async fn deliver_tx(&mut self, tx: Transaction) -> eyre::Result<Vec<Event>> {
        let tx = Arc::new(tx);
        tx.validate(tx.clone()).await?;
        tx.check(self.state.clone()).await?;

        let mut state_tx = self
            .state
            .try_begin_transaction()
            .expect("state Arc should not be referenced elsewhere");
        tx.execute(&mut state_tx).await?;

        Ok(state_tx.apply().1)
    }

    #[instrument(skip(self, tx_bytes))]
    pub async fn deliver_tx_bytes(&mut self, tx_bytes: &[u8]) -> eyre::Result<Vec<Event>> {
        let tx: Transaction = encoding::from_bytes(tx_bytes)?;
        self.deliver_tx(tx).await
    }

    #[instrument(skip(self))]
    pub async fn end_block(
        &mut self,
        end_block: &request::EndBlock,
    ) -> (Vec<Update>, Option<Params>, Vec<Event>) {
        let mut state_tx = self
            .state
            .try_begin_transaction()
            .expect("state Arc should not be referenced elsewhere");

        for component in &self.components {
            component.end_block(&mut state_tx, end_block).await;
        }

        let events = state_tx.apply().1;

        // TODO(xla): Implement val updates.
        let validator_updates = vec![];
        // TODO(xla): Implement consensus param updates.
        let consensus_param_updates = None;

        (validator_updates, consensus_param_updates, events)
    }

    #[instrument(skip(self, storage))]
    pub async fn commit(&mut self, storage: Storage) -> AppHash {
        // We need to extract the State we've built up to commit it.  Fill in a dummy state.
        let dummy_state = StateDelta::new(storage.latest_snapshot());
        let state = Arc::try_unwrap(std::mem::replace(&mut self.state, Arc::new(dummy_state)))
            .expect("we have exclusive ownership of the State at commit()");

        // Commit the pending writes, clearing the state.
        let jmt_root = storage
            .commit(state)
            .await
            .expect("must be able to successfully commit to storage");
        let app_hash: AppHash = jmt_root.into();

        tracing::debug!(?app_hash, "finished committing state");

        // Get the latest version of the state, now that we've committed it.
        self.state = Arc::new(StateDelta::new(storage.latest_snapshot()));

        app_hash
    }
}
