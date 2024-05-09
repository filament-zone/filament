#![deny(missing_docs)]

//! Filament Hub rollup.

use std::{env, str::FromStr};

use filament_hub_config::{ROLLUP_BATCH_NAMESPACE_RAW, ROLLUP_PROOF_NAMESPACE_RAW};
use sov_celestia_adapter::types::Namespace;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod celestia_rollup;
pub use celestia_rollup::*;

mod mock_rollup;
pub use mock_rollup::*;

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
            EnvFilter::from_str(&env::var("RUST_LOG").unwrap_or_else(|_| {
                "debug,hyper=info,risc0_zkvm=warn,sov_prover_storage_manager=info,jmt=info,sov_celestia_adapter=info,jsonrpsee_server=info"
                    .to_string()
            }))
            .unwrap(),
        )
        .init();
}
