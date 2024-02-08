//! This binary defines a CLI for interacting with the rollup.

#[cfg(feature = "celestia_da")]
use hub_rollup::celestia_rollup::CelestiaRollup as Rollup;
#[cfg(feature = "mock_da")]
use hub_rollup::mock_rollup::MockRollup as Rollup;
use hub_stf::runtime::RuntimeSubcommand;
use sov_modules_api::cli::{FileNameArg, JsonStringArg};
use sov_modules_rollup_blueprint::WalletBlueprint as _;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    Rollup::run_wallet::<
        RuntimeSubcommand<FileNameArg, _, _>,
        RuntimeSubcommand<JsonStringArg, _, _>,
    >()
    .await
}
