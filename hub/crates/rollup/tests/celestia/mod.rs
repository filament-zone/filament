use std::{collections::HashSet, ops::Range, time::Duration};

use borsh::BorshSerialize;
use filament_hub_stf::runtime;
use jsonrpsee::{
    core::client::{Subscription, SubscriptionClientT},
    rpc_params,
};
use rand::Rng;
use sov_celestia_adapter::verifier::CelestiaSpec;
use sov_cli::wallet_state::PrivateKeyAndAddress;
use sov_modules_api::{
    default_spec::DefaultSpec,
    transaction::{PriorityFeeBips, Transaction},
    Spec,
};
use sov_risc0_adapter::Risc0Verifier;
use sov_rollup_interface::da::DaSpec;
use sov_sequencer::utils::SimpleClient;

fn generate_dynamic_random_vectors(len_range: Range<usize>) -> Vec<Vec<u8>> {
    let mut rng = rand::thread_rng();
    let mut result = Vec::new();
    for length in len_range {
        let number_of_vectors = rng.gen_range(1..=3);

        let mut vectors_for_this_length = HashSet::new();

        while vectors_for_this_length.len() < number_of_vectors {
            let new_vector = (0..length).map(|_| rng.gen::<u8>()).collect::<Vec<u8>>();
            vectors_for_this_length.insert(new_vector);
        }

        result.extend(vectors_for_this_length.into_iter());
    }

    result
}

fn generate_call_message<S: Spec, Da: DaSpec>(
    len_range: Range<usize>,
) -> Vec<runtime::RuntimeCall<S, Da>> {
    let payloads = generate_dynamic_random_vectors(len_range);
    let mut messages = Vec::with_capacity(payloads.len());

    for payload in payloads {
        messages.push(runtime::RuntimeCall::value_setter(
            sov_value_setter::CallMessage::SetManyValues(payload),
        ));
    }

    messages
}

async fn submit_blobs_increasing_size<Da: DaSpec>() -> anyhow::Result<()> {
    // Purpose of this test to check that celestia adapter can process batches of various sizes.
    // This test submits batches in range of size, sequentially.
    // To minimize potential compression related issues,
    // each payload is generated randomly, and for each length there are 3 payloads
    //
    // This test requires appropriate rollup running on port 12345
    let blobs_payload_bytes_range = 1..10000;
    let token_deployer_data =
        std::fs::read_to_string("../../test-data/keys/token_deployer_private_key.json")
            .expect("Unable to read file to string");

    let token_deployer: PrivateKeyAndAddress<DefaultSpec<Risc0Verifier, Risc0Verifier>> =
        serde_json::from_str(&token_deployer_data).unwrap_or_else(|_| {
            panic!(
                "Unable to convert data {} to PrivateKeyAndAddress",
                &token_deployer_data
            )
        });

    let chain_id = 0;
    let max_priority_fee_bips = PriorityFeeBips::ZERO;
    let max_fee = 0;

    let messages = generate_call_message::<DefaultSpec<Risc0Verifier, Risc0Verifier>, Da>(
        blobs_payload_bytes_range,
    );
    println!("Generate {} messages", messages.len());

    let client = SimpleClient::new("localhost", 12345).await?;

    let mut slot_subscription: Subscription<u64> = client
        .ws()
        .subscribe(
            "ledger_subscribeSlots",
            rpc_params![],
            "ledger_unsubscribeSlots",
        )
        .await
        .unwrap();

    for (idx, message) in messages.into_iter().enumerate() {
        println!("Nonce {} . Going to submit message: {:?}", idx, message);
        let tx = Transaction::<DefaultSpec<Risc0Verifier, Risc0Verifier>>::new_signed_tx(
            &token_deployer.private_key,
            message.try_to_vec().unwrap(),
            chain_id,
            max_priority_fee_bips,
            max_fee,
            None,
            idx as u64,
        );
        client.send_transactions(&[tx]).await.unwrap();
        let slot = slot_subscription.next().await.unwrap().unwrap();
        println!("SLOT: {} received", slot);
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }

    Ok(())
}

#[tokio::test]
#[ignore = "Run manually"]
async fn test_celestia_increasing_blob_sizes() -> anyhow::Result<()> {
    // cargo test -p sov-demo-rollup --test all_tests celestia::test_celestia_increasing_blob_sizes
    // -- --nocapture --ignored
    submit_blobs_increasing_size::<CelestiaSpec>().await
}
