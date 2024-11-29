use std::{env, net::SocketAddr, str::FromStr as _};

use anyhow::Context as _;
use filament_hub_core::{
    campaign::{Campaign, Phase},
    criteria::{Criterion, CriterionCategory},
    CoreRpcClient,
};
use filament_hub_eth::Tx;
use filament_hub_stf::{genesis_config::GenesisPaths, RuntimeCall};
use futures::StreamExt;
use sov_kernels::basic::BasicKernelGenesisPaths;
use sov_mock_da::{BlockProducingConfig, MockAddress, MockDaConfig, MockDaSpec};
use sov_modules_api::{
    execution_mode::Native,
    macros::config_value,
    transaction::{PriorityFeeBips, TxDetails, UnsignedTransaction},
    Spec,
};
use sov_stf_runner::processes::RollupProverConfig;
use sov_test_utils::ApiClient;
use tracing_subscriber::{fmt, layer::SubscriberExt as _, util::SubscriberInitExt as _, EnvFilter};

use super::test_helpers::start_rollup;
use crate::test_helpers::read_eth_key;

type TestSpec = sov_modules_api::default_spec::DefaultSpec<
    sov_mock_zkvm::MockZkVerifier,
    sov_mock_zkvm::MockZkVerifier,
    Native,
>;

#[tokio::test(flavor = "multi_thread")]
async fn authenticate_tx_tests() -> Result<(), anyhow::Error> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::from_str(
                &env::var("RUST_LOG")
                    .unwrap_or_else(|_| "debug,hyper=info,jmt=info,risc0_zkvm=info,reqwest=info,tower_http=info,jsonrpsee-client=info,jsonrpsee-server=info,sqlx=warn".to_string()),
            )
            .unwrap(),
        )
        .init();
    let (rpc_port_tx, rpc_port_rx) = tokio::sync::oneshot::channel();
    let (rest_port_tx, rest_port_rx) = tokio::sync::oneshot::channel();

    let rollup_task = tokio::spawn(async {
        start_rollup(
            rpc_port_tx,
            rest_port_tx,
            GenesisPaths::from_dir("../../test-data/genesis/mock/"),
            BasicKernelGenesisPaths {
                chain_state: "../../test-data/genesis/mock/chain_state.json".into(),
            },
            RollupProverConfig::Skip,
            MockDaConfig {
                connection_string: "sqlite::memory:".to_string(),
                sender_address: MockAddress::new([0; 32]),
                finalization_blocks: 3,
                block_producing: BlockProducingConfig::OnSubmit,
                block_time_ms: 100_000,
            },
        )
        .await;
    });
    let rpc_port = rpc_port_rx.await.unwrap();
    let rest_port = rest_port_rx.await.unwrap();

    // If the rollup throws an error, return it and stop trying to send the transaction
    tokio::select! {
        err = rollup_task => err?,
        res = send_eth_tx(rpc_port, rest_port) => res?,
    }
    Ok(())
}

async fn send_eth_tx(
    rpc_address: SocketAddr,
    rest_address: SocketAddr,
) -> Result<(), anyhow::Error> {
    let (signing_key, address) = read_eth_key::<TestSpec>("signer.json")?;
    let user_address: <TestSpec as Spec>::Address = address;

    let chain_id = config_value!("CHAIN_ID");
    let nonce = 0;
    let max_priority_fee_bips = PriorityFeeBips::ZERO;
    let criteria = vec![Criterion {
        name: "Test Criterion".to_string(),
        category: CriterionCategory::Balance,
        parameters: Default::default(),
        weight: 1,
    }];

    let runtime_msg = {
        let msg = RuntimeCall::<TestSpec, MockDaSpec>::Core(filament_hub_core::CallMessage::<
            TestSpec,
        >::Draft {
            title: "".to_string(),
            description: "".to_string(),
            criteria: criteria.clone(),
            evictions: vec![],
        });
        borsh::to_vec(&msg)?
    };

    let unsigned_tx_bytes = borsh::to_vec(&UnsignedTransaction::<TestSpec>::new(
        runtime_msg.clone(),
        chain_id,
        max_priority_fee_bips,
        100,
        nonce,
        None,
    ))?;

    let signature = filament_hub_eth::sign(&signing_key, unsigned_tx_bytes)?;

    let tx: Tx<TestSpec> = Tx {
        signature: signature.to_vec(),
        verifying_key: signing_key.verifying_key().to_sec1_bytes().into_vec(),
        runtime_msg,
        nonce,
        details: TxDetails {
            max_priority_fee_bips,
            max_fee: 100,
            gas_limit: None,
            chain_id,
        },
    };

    let rpc_port = rpc_address.port();
    let rest_port = rest_address.port();
    let client = ApiClient::new(rpc_port, rest_port).await?;

    let mut slot_subscription = client
        .ledger
        .subscribe_slots()
        .await
        .context("Failed to subscribe to slots!")?;

    client
        .sequencer
        .publish_batch_with_serialized_txs(&[tx])
        .await?;
    // Wait until the rollup has processed the next slot
    let _slot_number = slot_subscription
        .next()
        .await
        .transpose()?
        .map(|slot| slot.number)
        .unwrap_or_default();

    let campaign_response = CoreRpcClient::<TestSpec>::rpc_get_campaign(&client.rpc, 0).await?;
    assert_eq!(
        campaign_response,
        Some(Campaign {
            campaigner: user_address,
            phase: Phase::Draft,
            title: "".to_string(),
            description: "".to_string(),
            criteria,
            evictions: vec![],
            delegates: vec![],

            indexer: None,
        }),
        "initialized campaign is incorrect"
    );

    Ok(())
}
