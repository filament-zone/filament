use filament_hub_rollup::CelestiaDemoRollup;
use filament_hub_stf::runtime::RuntimeSubcommand;
use sov_modules_api::cli::{FileNameArg, JsonStringArg};
use sov_modules_rollup_blueprint::WalletBlueprint;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    CelestiaDemoRollup::run_wallet::<
        RuntimeSubcommand<FileNameArg, _, _>,
        RuntimeSubcommand<JsonStringArg, _, _>,
    >()
    .await
}