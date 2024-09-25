//! This binary defines a cli wallet for interacting
//! with the rollup.

#[cfg(all(feature = "celestia_da", not(feature = "mock_da")))]
use filament_hub_rollup::celestia_rollup::CelestiaRollup as StarterRollup;
#[cfg(all(feature = "mock_da", not(feature = "celestia_da")))]
use filament_hub_rollup::mock_rollup::MockRollup as StarterRollup;
use filament_hub_stf::runtime::RuntimeSubcommand;
use sov_modules_api::cli::{FileNameArg, JsonStringArg};
use sov_modules_rollup_blueprint::WalletBlueprint as _;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    StarterRollup::run_wallet::<
        RuntimeSubcommand<FileNameArg, _, _>,
        RuntimeSubcommand<JsonStringArg, _, _>,
    >()
    .await
}
