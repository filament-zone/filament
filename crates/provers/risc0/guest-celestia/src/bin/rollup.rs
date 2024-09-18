// TODO: Rename this file to change the name of this method from METHOD_NAME

#![no_main]

use filament_hub_stf::{runtime::Runtime, StfVerifier};
use sov_celestia_adapter::{types::Namespace, verifier::CelestiaVerifier};
use sov_kernels::basic::BasicKernel;
use sov_mock_zkvm::{MockZkGuest, MockZkVerifier};
use sov_modules_api::{default_spec::DefaultSpec, execution_mode::Zk};
use sov_modules_stf_blueprint::StfBlueprint;
use sov_risc0_adapter::{guest::Risc0Guest, Risc0Verifier};
use sov_state::ZkStorage;

/// The namespace for the rollup on Celestia. Must be kept in sync with the "rollup/src/lib.rs"
const ROLLUP_BATCH_NAMESPACE: Namespace = Namespace::const_v0(*b"sov-test-b");
const ROLLUP_PROOF_NAMESPACE: Namespace = Namespace::const_v0(*b"sov-test-p");

risc0_zkvm::guest::entry!(main);

pub fn main() {
    let guest = Risc0Guest::new();
    let storage = ZkStorage::new();
    let stf: StfBlueprint<
        DefaultSpec<Risc0Verifier, MockZkVerifier, Zk>,
        _,
        Runtime<_, _>,
        BasicKernel<_, _>,
    > = StfBlueprint::new();

    let stf_verifier = StfVerifier::<_, _, _, _, Risc0Guest, MockZkGuest>::new(
        stf,
        CelestiaVerifier {
            rollup_batch_namespace: ROLLUP_BATCH_NAMESPACE,
            rollup_proof_namespace: ROLLUP_PROOF_NAMESPACE,
        },
    );
    stf_verifier
        .run_block(guest, storage)
        .expect("Prover must be honest");
}
