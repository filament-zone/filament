use demo_stf::runtime::RuntimeCall;
use ethers_core::abi::Address;
use sov_mock_da::MockDaSpec;
use sov_modules_api::transaction::{Transaction, UnsignedTransaction};
use sov_modules_macros::config_value;
use sov_test_utils::{TestSpec, TEST_DEFAULT_MAX_FEE, TEST_DEFAULT_MAX_PRIORITY_FEE};

use super::test_client::TestClient;
use crate::{
    evm::evm_test_helper::{self},
    test_helpers::{get_appropriate_rollup_prover_config, read_private_keys},
};

#[tokio::test(flavor = "multi_thread")]
async fn test_evm_account_abstraction() {
    let chain_id = config_value!("CHAIN_ID");
    let finalization_blocks = 0;
    let rollup_prover_config = get_appropriate_rollup_prover_config();
    let (rollup_task, rpc_port, rest_port) =
        evm_test_helper::start_node(rollup_prover_config, finalization_blocks).await;

    let (test_client, from_addr) = evm_test_helper::create_test_client(
        rpc_port,
        rest_port,
        chain_id,
        // This will produce an evm key that doesn't exist in rollup accounts-genesis so we have to
        // register the credentials in the rollup.
        "0x90cb5be9e2c125d84af44f19a4e6e36af359bd47b41577aedbe8aa24313bbd40",
    )
    .await;

    // Before executing the evm checks we need to insert the credentials in the `Accounts`.
    send_insert_credentials(&test_client, from_addr, chain_id).await;
    // Execute the evm tests.
    execute_evm_tests(&test_client).await.unwrap();

    rollup_task.abort();
}

async fn send_insert_credentials(test_client: &TestClient, from_addr: Address, chain_id: u64) {
    let tx = vec![create_insert_credentials(from_addr, chain_id)];
    test_client
        .send_transactions_and_wait_slot(&tx)
        .await
        .unwrap();
}

fn create_insert_credentials(from_addr: Address, chain_id: u64) -> Transaction<TestSpec> {
    let nonce = 0;
    let key_and_address = read_private_keys::<TestSpec>("tx_signer_private_key.json");
    let key = key_and_address.private_key;

    let mut credentials = [0; 32];
    credentials[12..].copy_from_slice(&from_addr.to_fixed_bytes());

    let msg = RuntimeCall::<TestSpec, MockDaSpec>::Accounts(
        sov_accounts::CallMessage::InsertCredentialId(credentials.into()),
    );

    let max_priority_fee_bips = TEST_DEFAULT_MAX_PRIORITY_FEE;
    let max_fee = TEST_DEFAULT_MAX_FEE;
    let gas_limit = None;
    Transaction::<TestSpec>::new_signed_tx(
        &key,
        UnsignedTransaction::new(
            borsh::to_vec(&msg).unwrap(),
            chain_id,
            max_priority_fee_bips,
            max_fee,
            nonce,
            gas_limit,
        ),
    )
}

async fn execute_evm_tests(client: &TestClient) -> Result<(), Box<dyn std::error::Error>> {
    let nonce = client.eth_get_transaction_count(client.from_addr).await;
    assert_eq!(0, nonce);

    // Balance should be > 0 in genesis
    let balance = client.eth_get_balance(client.from_addr).await;
    assert!(balance > ethereum_types::U256::zero());

    let mut slot_subscription = client.subscribe_for_slots().await?;
    let contract_address =
        evm_test_helper::deploy_contract_check(client, &mut slot_subscription).await?;

    // Nonce should be 1 after the deploy
    let nonce = client.eth_get_transaction_count(client.from_addr).await;
    assert_eq!(1, nonce);

    let set_arg = 923;
    evm_test_helper::set_value_check(client, &mut slot_subscription, contract_address, set_arg)
        .await?;

    Ok(())
}
