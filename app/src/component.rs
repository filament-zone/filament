use async_trait::async_trait;
use penumbra_storage::StateWrite;
use pulzaar_chain::genesis::AppState;
use tendermint::abci::request::{BeginBlock, EndBlock};

pub mod accounts;
pub mod staking;

pub use staking::Staking;

pub enum Component {
    Staking(Staking),
}

/// A component to be called for chain and block related ABCI calls.
#[async_trait]
pub trait ABCIComponent<S>: Send + Sync + 'static
where
    S: StateWrite,
{
    /// * Called once upon genesis.
    /// * If ResponseInitChain.Validators is empty, the initial validator set will be the
    ///   RequestInitChain.Validators
    /// * If ResponseInitChain.Validators is not empty, it will be the initial validator set
    ///   (regardless of what is in RequestInitChain.Validators).
    /// * This allows the app to decide if it wants to accept the initial validator set proposed by
    ///   tendermint (ie. in the genesis file), or if it wants to use a different one (perhaps
    ///   computed based on some application specific information in the genesis file).
    ///
    /// <https://github.com/tendermint/tendermint/blob/main/spec/abci/abci.md#initchain>
    async fn init_chain(&self, state: &mut S, app_state: &AppState);

    /// * Signals the beginning of a new block.
    /// * Called prior to any `DeliverTx` method calls.
    /// * The header contains the height, timestamp, and more - it exactly matches the Tendermint
    ///   block header. We may seek to generalize this in the future.
    /// * The `LastCommitInfo` and `ByzantineValidators` can be used to determine rewards and
    ///   punishments for the validators.
    ///
    /// <https://github.com/tendermint/tendermint/blob/main/spec/abci/abci.md#initchain>
    async fn begin_block(&self, state: &mut S, begin_blocke: &BeginBlock);

    /// * Signals the end of a block.
    /// * Called after all the transactions for the current block have been delivered, prior to the
    ///   block's `Commit` message.
    /// * Optional `validator_updates` triggered by block `H`. These updates affect validation for
    ///   blocks `H+1`, `H+2`, and `H+3`.
    /// * Heights following a validator update are affected in the following way:
    /// * `H+1`: `NextValidatorsHash` includes the new `validator_updates` value.
    /// * `H+2`: The validator set change takes effect and `ValidatorsHash` is updated.
    /// * `H+3`: `LastCommitInfo` is changed to include the altered validator set.
    /// * `consensus_param_updates` returned for block `H` apply to the consensus params for block `H+1`. For more information on the consensus parameters, see the [application spec entry on consensus parameters](https://github.com/tendermint/tendermint/blob/main/spec/abci/apps.md#consensus-parameters).
    ///
    /// <https://github.com/tendermint/tendermint/blob/main/spec/abci/abci.md#endblock>
    async fn end_block(&self, state: &mut S, end_block: &EndBlock);
}
