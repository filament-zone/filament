#![no_main]
sp1_zkvm::entrypoint!(main);

use filament_hub_config::{ROLLUP_BATCH_NAMESPACE_RAW, ROLLUP_PROOF_NAMESPACE_RAW};
use filament_hub_stf::{runtime::Runtime, StfVerifier};
use sov_celestia_adapter::{types::Namespace, verifier::CelestiaVerifier};
use sov_kernels::basic::BasicKernel;
use sov_mock_zkvm::{MockZkGuest, MockZkVerifier};
use sov_modules_api::{default_spec::DefaultSpec, execution_mode::Zk};
use sov_modules_stf_blueprint::StfBlueprint;
use sov_sp1_adapter::{guest::SP1Guest, SP1Verifier};
use sov_state::ZkStorage;

// The rollup stores its data in the namespace b"sov-test" on Celestia
const ROLLUP_BATCH_NAMESPACE: Namespace = Namespace::const_v0(ROLLUP_BATCH_NAMESPACE_RAW);
const ROLLUP_PROOF_NAMESPACE: Namespace = Namespace::const_v0(ROLLUP_PROOF_NAMESPACE_RAW);

pub fn main() {
    let guest = SP1Guest::new();
    let storage = ZkStorage::new();
    let stf: StfBlueprint<
        DefaultSpec<SP1Verifier, MockZkVerifier, Zk>,
        _,
        Runtime<_, _>,
        BasicKernel<_, _>,
    > = StfBlueprint::new();

    let stf_verifier = StfVerifier::<_, _, _, _, SP1Guest, MockZkGuest>::new(
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
