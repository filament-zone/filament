#![deny(missing_docs)]
#![doc = include_str!("../README.md")]

use std::{env, str::FromStr};

use filament_hub_config::{ROLLUP_BATCH_NAMESPACE_RAW, ROLLUP_PROOF_NAMESPACE_RAW};
use sov_celestia_adapter::types::Namespace;

mod mock_rollup;

pub use mock_rollup::*;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod celestia_rollup;

pub use celestia_rollup::*;
use sov_modules_rollup_blueprint::DEFAULT_SOV_ROLLUP_LOGGING;

mod eth;

/// The rollup stores its data in the namespace b"sov-test" on Celestia
/// You can change this constant to point your rollup at a different namespace
pub const ROLLUP_BATCH_NAMESPACE: Namespace = Namespace::const_v0(ROLLUP_BATCH_NAMESPACE_RAW);

/// The rollup stores the zk proofs in the namespace b"sov-test-p" on Celestia.
pub const ROLLUP_PROOF_NAMESPACE: Namespace = Namespace::const_v0(ROLLUP_PROOF_NAMESPACE_RAW);

/// Default initialization of logging
pub fn initialize_logging() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::from_str(
                &env::var("RUST_LOG").unwrap_or_else(|_| DEFAULT_SOV_ROLLUP_LOGGING.to_string()),
            )
            .unwrap(),
        )
        .init();

    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        tracing_panic::panic_hook(panic_info);
        prev_hook(panic_info);
    }));
}
