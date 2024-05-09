use sov_accounts::Response;
use sov_mock_da::{MockAddress, MockBlock, MockDaSpec, MOCK_SEQUENCER_DA_ADDRESS};
use sov_modules_api::{batch::BatchWithId, PrivateKey, PublicKey, Spec, WorkingSet};
use sov_modules_stf_blueprint::{SequencerOutcome, SlashingReason, StfBlueprint, TxEffect};
use sov_rollup_interface::{
    da::RelevantBlobs,
    services::da::SlotData,
    stf::StateTransitionFunction,
    storage::HierarchicalStorageManager,
};
use sov_test_utils::{
    bank_data::get_default_token_id,
    has_tx_events,
    new_test_blob_from_batch,
    TestHasher,
    TestSpec,
};

use super::{
    create_genesis_config_for_tests,
    create_storage_manager_for_tests,
    read_private_keys,
    RuntimeTest,
};
use crate::{
    runtime::Runtime,
    tests::{
        da_simulation::{
            simulate_da_with_bad_nonce,
            simulate_da_with_bad_serialization,
            simulate_da_with_bad_sig,
            simulate_da_with_revert_msg,
        },
        StfBlueprintTest,
    },
};

// Assume there was a proper address and we converted it to bytes already.
const SEQUENCER_DA_ADDRESS: [u8; 32] = [1; 32];

#[test]
fn test_tx_revert() {
    // Test checks:
    //  - Batch is successfully applied even with incorrect txs
    //  - Nonce for bad transactions has increased

    let tempdir = tempfile::tempdir().unwrap();

    let config = create_genesis_config_for_tests();
    let sequencer_rollup_address = config.runtime.sequencer_registry.seq_rollup_address;

    let genesis_block = MockBlock::default();
    let block_1 = genesis_block.next_mock();
    let admin_key = read_private_keys::<TestSpec>().token_deployer.private_key;
    let admin_address: <TestSpec as Spec>::Address = admin_key.to_address();

    let storage = {
        let mut storage_manager = create_storage_manager_for_tests(tempdir.path());
        let stf: StfBlueprintTest = StfBlueprint::new();

        let (stf_state, ledger_state) = storage_manager
            .create_state_for(genesis_block.header())
            .unwrap();
        let (genesis_root, stf_state) = stf.init_chain(stf_state, config);
        storage_manager
            .save_change_set(genesis_block.header(), stf_state, ledger_state.into())
            .unwrap();

        let txs = simulate_da_with_revert_msg(admin_key.clone());
        let blob = new_test_blob_from_batch(
            BatchWithId { txs, id: [0; 32] },
            &MOCK_SEQUENCER_DA_ADDRESS,
            [0; 32],
        );

        let mut relevant_blobs = RelevantBlobs {
            proof_blobs: Default::default(),
            batch_blobs: vec![blob],
        };

        let (stf_state, ledger_state) = storage_manager.create_state_for(block_1.header()).unwrap();
        let apply_block_result = stf.apply_slot(
            &genesis_root,
            stf_state,
            Default::default(),
            &block_1.header,
            &block_1.validity_cond,
            relevant_blobs.as_iters(),
        );

        assert_eq!(1, apply_block_result.batch_receipts.len());
        let apply_blob_outcome = apply_block_result.batch_receipts[0].clone();

        assert_eq!(
            SequencerOutcome::Rewarded(0),
            apply_blob_outcome.inner,
            "Unexpected outcome: Batch execution should have succeeded",
        );

        let txn_receipts = apply_block_result.batch_receipts[0].tx_receipts.clone();
        // 3 transactions
        // create 1000 tokens
        // transfer 15 tokens
        // transfer 5000 tokens // this should be reverted
        assert_eq!(txn_receipts[0].receipt, TxEffect::Successful);
        assert_eq!(txn_receipts[1].receipt, TxEffect::Successful);
        assert_eq!(txn_receipts[2].receipt, TxEffect::Reverted);

        storage_manager
            .save_change_set(
                block_1.header(),
                apply_block_result.change_set,
                ledger_state.into(),
            )
            .unwrap();
        let (storage, _) = storage_manager
            .create_state_after(block_1.header())
            .unwrap();
        storage
    };

    // Checks on storage after execution
    {
        let runtime = &mut Runtime::<TestSpec, MockDaSpec>::default();
        let mut working_set = WorkingSet::new(storage);
        let resp = runtime
            .bank
            .balance_of(
                None,
                admin_address,
                get_default_token_id::<TestSpec>(&admin_address),
                &mut working_set,
            )
            .unwrap();

        assert_eq!(resp.amount, Some(985));

        let resp = runtime
            .sequencer_registry
            .sequencer_address(
                MockAddress::from(MOCK_SEQUENCER_DA_ADDRESS),
                &mut working_set,
            )
            .unwrap();
        // Sequencer is not excluded from the list of allowed!
        assert_eq!(Some(sequencer_rollup_address), resp.address);

        let nonce = match runtime
            .accounts
            .get_account(
                admin_key.pub_key().secure_hash::<TestHasher>(),
                &mut working_set,
            )
            .unwrap()
        {
            Response::AccountExists { nonce, .. } => nonce,
            Response::AccountEmpty => 0,
        };

        // with 3 transactions, the final nonce should be 3
        // 0 -> 1
        // 1 -> 2
        // 2 -> 3
        // The minter account should have its nonce increased for 3 transactions
        assert_eq!(3, nonce);
    }
}

#[test]
fn test_tx_bad_signature() {
    let tempdir = tempfile::tempdir().unwrap();
    let path = tempdir.path();

    let config = create_genesis_config_for_tests();

    let genesis_block = MockBlock::default();
    let block_1 = genesis_block.next_mock();
    let admin_key = read_private_keys::<TestSpec>().token_deployer.private_key;
    let storage = {
        let mut storage_manager = create_storage_manager_for_tests(path);
        let stf: StfBlueprintTest = StfBlueprint::new();
        let (stf_state, ledger_state) = storage_manager
            .create_state_for(genesis_block.header())
            .unwrap();
        let (genesis_root, stf_state) = stf.init_chain(stf_state, config);
        storage_manager
            .save_change_set(genesis_block.header(), stf_state, ledger_state.into())
            .unwrap();

        let txs = simulate_da_with_bad_sig(admin_key.clone());

        let blob = new_test_blob_from_batch(
            BatchWithId { txs, id: [0; 32] },
            &MOCK_SEQUENCER_DA_ADDRESS,
            [0; 32],
        );

        let mut relevant_blobs = RelevantBlobs {
            proof_blobs: Default::default(),
            batch_blobs: vec![blob],
        };

        let (stf_state, ledger_state) = storage_manager.create_state_for(block_1.header()).unwrap();
        let apply_block_result = stf.apply_slot(
            &genesis_root,
            stf_state,
            Default::default(),
            &block_1.header,
            &block_1.validity_cond,
            relevant_blobs.as_iters(),
        );

        assert_eq!(1, apply_block_result.batch_receipts.len());
        let apply_blob_outcome = apply_block_result.batch_receipts[0].clone();

        assert_eq!(
            SequencerOutcome::Slashed(
                SlashingReason::StatelessVerificationFailed,
            ),
            apply_blob_outcome.inner,
            "Unexpected outcome: Stateless verification should have failed due to invalid signature"
        );

        // The batch receipt contains no events.
        assert!(!has_tx_events(&apply_blob_outcome));
        storage_manager
            .save_change_set(
                block_1.header(),
                apply_block_result.change_set,
                ledger_state.into(),
            )
            .unwrap();
        let (storage, _) = storage_manager
            .create_state_after(block_1.header())
            .unwrap();
        storage
    };

    {
        let runtime = &mut Runtime::<TestSpec, MockDaSpec>::default();
        let mut working_set = WorkingSet::new(storage);
        let nonce = match runtime
            .accounts
            .get_account(
                admin_key.pub_key().secure_hash::<TestHasher>(),
                &mut working_set,
            )
            .unwrap()
        {
            Response::AccountExists { nonce, .. } => nonce,
            Response::AccountEmpty => 0,
        };
        assert_eq!(0, nonce);
    }
}

#[test]
fn test_tx_bad_nonce() {
    let tempdir = tempfile::tempdir().unwrap();
    let path = tempdir.path();

    let config = create_genesis_config_for_tests();
    let genesis_block = MockBlock::default();
    let block_1 = genesis_block.next_mock();
    let admin_key = read_private_keys::<TestSpec>().token_deployer.private_key;
    {
        let mut storage_manager = create_storage_manager_for_tests(path);
        let stf: StfBlueprintTest = StfBlueprint::new();
        let (stf_state, ledger_state) = storage_manager
            .create_state_for(genesis_block.header())
            .unwrap();
        let (genesis_root, stf_state) = stf.init_chain(stf_state, config);
        storage_manager
            .save_change_set(genesis_block.header(), stf_state, ledger_state.into())
            .unwrap();
        let txs = simulate_da_with_bad_nonce(admin_key);

        let blob = new_test_blob_from_batch(
            BatchWithId { txs, id: [0; 32] },
            &MOCK_SEQUENCER_DA_ADDRESS,
            [0; 32],
        );

        let mut relevant_blobs = RelevantBlobs {
            proof_blobs: Default::default(),
            batch_blobs: vec![blob],
        };

        let (stf_state, _ledger_state) =
            storage_manager.create_state_for(block_1.header()).unwrap();
        let apply_block_result = stf.apply_slot(
            &genesis_root,
            stf_state,
            Default::default(),
            &block_1.header,
            &block_1.validity_cond,
            relevant_blobs.as_iters(),
        );

        assert_eq!(1, apply_block_result.batch_receipts.len());
        let tx_receipts = apply_block_result.batch_receipts[0].tx_receipts.clone();
        // Bad nonce means that the transaction has to be reverted
        assert_eq!(tx_receipts[0].receipt, TxEffect::Duplicate);

        // We don't slash the sequencer for a bad nonce, since the nonce change might have
        // happened while the transaction was in-flight. However, we do *penalize* the sequencer
        // in this case.
        // We're asserting that here to track if the logic changes
        let sequencer_outcome = apply_block_result.batch_receipts[0].inner.clone();
        match sequencer_outcome {
            SequencerOutcome::Rewarded(amount) => assert_eq!(amount, 0), // If the gas price is
            // zero, the sequencer
            // might not be
            // rewarded or
            // penalized.
            SequencerOutcome::Penalized(amount) => assert!(amount > 0),
            _ => panic!("Sequencer should have been penalized"),
        }
    }
}

#[test]
fn test_tx_bad_serialization() {
    let tempdir = tempfile::tempdir().unwrap();
    let path = tempdir.path();

    let config = create_genesis_config_for_tests();
    let sequencer_rollup_address = config.runtime.sequencer_registry.seq_rollup_address;

    let genesis_block = MockBlock::default();
    let block_1 = genesis_block.next_mock();
    let mut storage_manager = create_storage_manager_for_tests(path);
    let admin_key = read_private_keys::<TestSpec>().token_deployer.private_key;

    let (genesis_root, sequencer_balance_before) = {
        let stf: StfBlueprintTest = StfBlueprint::new();

        let (stf_state, ledger_state) = storage_manager
            .create_state_for(genesis_block.header())
            .unwrap();
        let (genesis_root, stf_state) = stf.init_chain(stf_state, config);
        storage_manager
            .save_change_set(genesis_block.header(), stf_state, ledger_state.into())
            .unwrap();

        let balance = {
            let (stf_state, _) = storage_manager
                .create_state_after(genesis_block.header())
                .unwrap();
            let runtime: RuntimeTest = Runtime::default();
            let mut working_set = WorkingSet::<TestSpec>::new(stf_state.clone());

            let coins = runtime
                .sequencer_registry
                .get_coins_to_lock(&mut working_set);

            runtime
                .bank
                .get_balance_of(&sequencer_rollup_address, coins.token_id, &mut working_set)
                .unwrap()
        };
        (genesis_root, balance)
    };

    let storage = {
        let stf: StfBlueprintTest = StfBlueprint::new();

        let txs = simulate_da_with_bad_serialization(admin_key.clone());
        let blob = new_test_blob_from_batch(
            BatchWithId { txs, id: [0; 32] },
            &MOCK_SEQUENCER_DA_ADDRESS,
            [0; 32],
        );

        let mut relevant_blobs = RelevantBlobs {
            proof_blobs: Default::default(),
            batch_blobs: vec![blob],
        };

        let (storage, ledger_state) = storage_manager.create_state_for(block_1.header()).unwrap();
        let apply_block_result = stf.apply_slot(
            &genesis_root,
            storage,
            Default::default(),
            &block_1.header,
            &block_1.validity_cond,
            relevant_blobs.as_iters(),
        );

        assert_eq!(1, apply_block_result.batch_receipts.len());
        let apply_blob_outcome = apply_block_result.batch_receipts[0].clone();

        assert_eq!(
            SequencerOutcome::Slashed (
                 SlashingReason::InvalidTransactionEncoding ,
            ),
            apply_blob_outcome.inner,
            "Unexpected outcome: Stateless verification should have failed due to invalid signature"
        );

        // The batch receipt contains no events.
        assert!(!has_tx_events(&apply_blob_outcome));
        storage_manager
            .save_change_set(
                block_1.header(),
                apply_block_result.change_set,
                ledger_state.into(),
            )
            .unwrap();
        let (storage, _) = storage_manager
            .create_state_after(block_1.header())
            .unwrap();
        storage
    };

    {
        let runtime = &mut Runtime::<TestSpec, MockDaSpec>::default();
        let mut working_set = WorkingSet::new(storage);

        // Sequencer is not in the list of allowed sequencers

        let allowed_sequencer = runtime
            .sequencer_registry
            .sequencer_address(MockAddress::from(SEQUENCER_DA_ADDRESS), &mut working_set)
            .unwrap();
        assert!(allowed_sequencer.address.is_none());

        // Balance of sequencer is not increased
        let coins = runtime
            .sequencer_registry
            .get_coins_to_lock(&mut working_set);
        let sequencer_balance_after = runtime
            .bank
            .get_balance_of(&sequencer_rollup_address, coins.token_id, &mut working_set)
            .unwrap();
        assert_eq!(sequencer_balance_before, sequencer_balance_after);
    }
}
