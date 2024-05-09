use std::vec;

use sov_mock_da::MockBlock;
use sov_modules_api::batch::BatchWithId;
use sov_modules_stf_blueprint::StfBlueprint;
use sov_rollup_interface::{
    da::RelevantBlobs,
    services::da::SlotData,
    stf::StateTransitionFunction,
    storage::HierarchicalStorageManager,
};
use sov_test_utils::{new_test_blob_from_batch, TestSpec};

use crate::tests::{
    create_genesis_config_for_tests,
    create_storage_manager_for_tests,
    da_simulation::simulate_da,
    read_private_keys,
    StfBlueprintTest,
};

#[test]
fn test_sequencer_unknown_sequencer() {
    let tempdir = tempfile::tempdir().unwrap();
    let path = tempdir.path();

    let mut config = create_genesis_config_for_tests();
    config.runtime.sequencer_registry.is_preferred_sequencer = false;

    let genesis_block = MockBlock::default();
    let block_1 = genesis_block.next_mock();

    let mut storage_manager = create_storage_manager_for_tests(path);
    let stf: StfBlueprintTest = StfBlueprint::new();
    let (stf_state, ledger_state) = storage_manager
        .create_state_for(genesis_block.header())
        .unwrap();
    let (genesis_root, stf_state) = stf.init_chain(stf_state, config);
    storage_manager
        .save_change_set(genesis_block.header(), stf_state, ledger_state.into())
        .unwrap();

    let some_sequencer: [u8; 32] = [121; 32];

    let private_key = read_private_keys::<TestSpec>().tx_signer.private_key;
    let txs = simulate_da(private_key);
    let blob = new_test_blob_from_batch(BatchWithId { txs, id: [0; 32] }, &some_sequencer, [0; 32]);

    let mut relevant_blobs = RelevantBlobs {
        proof_blobs: Default::default(),
        batch_blobs: vec![blob],
    };

    let (stf_state, _ledger_state) = storage_manager.create_state_for(block_1.header()).unwrap();

    let apply_block_result = stf.apply_slot(
        &genesis_root,
        stf_state,
        Default::default(),
        &block_1.header,
        &block_1.validity_cond,
        relevant_blobs.as_iters(),
    );

    // The sequencer isn't registered, so the blob should be ignored.
    assert_eq!(0, apply_block_result.batch_receipts.len());
}
