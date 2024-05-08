use std::env;

use filament_hub_rollup::MockDemoRollup;
use serde::Serialize;
use sov_cli::wallet_state::PrivateKeyAndAddress;
use sov_mock_da::{MockAddress, MockBlock, MockDaService};
use sov_modules_api::{CryptoSpec, Spec};
use sov_rollup_interface::services::da::DaService;
use sov_test_utils::{bank_data::BankMessageGenerator, MessageGenerator};

use crate::test_helpers::read_private_keys;

type S = sov_modules_api::default_spec::DefaultSpec<
    sov_risc0_adapter::Risc0Verifier,
    sov_mock_zkvm::MockZkVerifier,
>;
type DefaultPublicKey = <<S as Spec>::CryptoSpec as CryptoSpec>::PublicKey;

#[derive(Serialize)]
struct AccountsData {
    pub_keys: Vec<DefaultPublicKey>,
}

const DEFAULT_BLOCKS: u64 = 10;
const DEFAULT_TXNS_PER_BLOCK: u64 = 100;

pub async fn get_blocks_from_da() -> anyhow::Result<Vec<MockBlock>> {
    let txns_per_block = match env::var("TXNS_PER_BLOCK") {
        Ok(txns_per_block) => txns_per_block.parse::<u64>()?,
        Err(_) => {
            println!("TXNS_PER_BLOCK not set, using default");
            DEFAULT_TXNS_PER_BLOCK
        },
    };

    let block_cnt = match env::var("BLOCKS") {
        Ok(block_cnt_str) => block_cnt_str.parse::<u64>()?,
        Err(_) => {
            println!("BLOCKS not set, using default");
            DEFAULT_BLOCKS
        },
    };

    let da_service = MockDaService::new(MockAddress::default());
    let mut blocks = vec![];

    let private_key_and_address: PrivateKeyAndAddress<S> =
        read_private_keys::<S>("minter_private_key.json");

    let (create_token_message_gen, transfer_message_gen) =
        BankMessageGenerator::generate_token_and_random_transfers(
            txns_per_block,
            private_key_and_address.private_key,
        );
    let blob = create_token_message_gen.create_blobs::<<MockDemoRollup as sov_modules_rollup_blueprint::RollupBlueprint>::NativeRuntime>();
    let fee = da_service.estimate_fee(blob.len()).await.unwrap();
    da_service.send_transaction(&blob, fee).await.unwrap();
    let block1 = da_service.get_block_at(1).await.unwrap();
    blocks.push(block1);

    for i in 0..block_cnt {
        let blob = transfer_message_gen.create_blobs::<<MockDemoRollup as sov_modules_rollup_blueprint::RollupBlueprint>::NativeRuntime>();
        let fee = da_service.estimate_fee(blob.len()).await.unwrap();
        da_service.send_transaction(&blob, fee).await.unwrap();
        let blocki = da_service.get_block_at(2 + i).await.unwrap();
        blocks.push(blocki);
    }

    Ok(blocks)
}
