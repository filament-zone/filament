use std::net::SocketAddr;

use borsh::BorshSerialize;
use hub_rollup::mock_rollup::MockRollup;
use hub_stf::{genesis_config::GenesisPaths, RuntimeCall};
use jsonrpsee::{
    core::client::{Subscription, SubscriptionClientT},
    rpc_params,
};
use sov_mock_da::{MockAddress, MockDaConfig, MockDaSpec};
use sov_modules_api::{
    default_context::DefaultContext,
    default_signature::private_key::DefaultPrivateKey,
    transaction::Transaction,
    PrivateKey,
};
use sov_modules_rollup_blueprint::RollupBlueprint;
use sov_modules_stf_blueprint::kernels::basic::{
    BasicKernelGenesisConfig,
    BasicKernelGenesisPaths,
};
use sov_sequencer::utils::SimpleClient;
use sov_stf_runner::{
    ProverServiceConfig,
    RollupConfig,
    RollupProverConfig,
    RpcConfig,
    RunnerConfig,
    StorageConfig,
};
use tokio::sync::oneshot;

#[tokio::test]
async fn outpost_tests() -> Result<(), anyhow::Error> {
    let (port_tx, port_rx) = tokio::sync::oneshot::channel();

    let rollup_task = tokio::spawn(async {
        start_rollup(
            port_tx,
            GenesisPaths::from_dir("../../test-data/genesis/mock/"),
            BasicKernelGenesisPaths {
                chain_state: "../../test-data/genesis/mock/chain_state.json".into(),
            },
            RollupProverConfig::Execute,
        )
        .await;
    });

    let port = port_rx.await.unwrap();

    // If the rollup throws an error, return it and stop trying to send the transaction
    tokio::select! {
        err = rollup_task => err?,
        res = register_outpost(port) => res?,
    }
    Ok(())
}

async fn register_outpost(rpc_address: SocketAddr) -> Result<(), anyhow::Error> {
    let key = DefaultPrivateKey::generate();

    let msg = RuntimeCall::<DefaultContext, MockDaSpec>::outpost_registry(
        fila_outposts::CallMessage::Register {
            chain_id: "neutron-1".to_owned(),
        },
    );
    let chain_id = 0;
    let gas_tip = 0;
    let gas_limit = 0;
    let max_gas_price = None;
    let nonce = 0;
    let tx = Transaction::<DefaultContext>::new_signed_tx(
        &key,
        msg.try_to_vec().unwrap(),
        chain_id,
        gas_tip,
        gas_limit,
        max_gas_price,
        nonce,
    );

    let port = rpc_address.port();
    let client = SimpleClient::new("localhost", port).await?;

    let mut slot_processed_subscription: Subscription<u64> = client
        .ws()
        .subscribe(
            "ledger_subscribeSlots",
            rpc_params![],
            "ledger_unsubscribeSlots",
        )
        .await?;

    client.send_transaction(tx).await?;

    // Wait until the rollup has processed the next slot
    let _ = slot_processed_subscription.next().await;

    let outpost_response = fila_outposts::OutpostRegistryRpcClient::<DefaultContext>::get_outpost(
        client.http(),
        "neutron-1".to_owned(),
    )
    .await?;

    assert!(outpost_response.is_some());

    Ok(())
}

async fn start_rollup(
    rpc_reporting_channel: oneshot::Sender<SocketAddr>,
    rt_genesis_paths: GenesisPaths,
    kernel_genesis_paths: BasicKernelGenesisPaths,
    rollup_prover_config: RollupProverConfig,
) {
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_path = temp_dir.path();

    let rollup_config = RollupConfig {
        storage: StorageConfig {
            path: temp_path.to_path_buf(),
        },
        runner: RunnerConfig {
            start_height: 1,
            da_polling_interval_ms: 1000,
            rpc_config: RpcConfig {
                bind_host: "127.0.0.1".into(),
                bind_port: 0,
            },
        },
        da: MockDaConfig {
            sender_address: MockAddress::from([0; 32]),
            finalization_blocks: 0,
            wait_attempts: 10,
        },
        prover_service: ProverServiceConfig {
            aggregated_proof_block_jump: 1,
        },
    };

    let mock_rollup = MockRollup {};

    let kernel_genesis = BasicKernelGenesisConfig {
        chain_state: serde_json::from_str(
            &std::fs::read_to_string(&kernel_genesis_paths.chain_state)
                .expect("Failed to read chain_state genesis config"),
        )
        .expect("Failed to parse chain_state genesis config"),
    };

    let rollup = mock_rollup
        .create_new_rollup(
            &rt_genesis_paths,
            kernel_genesis,
            rollup_config,
            rollup_prover_config,
        )
        .await
        .unwrap();
    rollup
        .run_and_report_rpc_port(Some(rpc_reporting_channel))
        .await
        .unwrap();

    // Close the tempdir explicitly to ensure that rustc doesn't see that it's unused and drop it
    // unexpectedly
    temp_dir.close().unwrap();
}
