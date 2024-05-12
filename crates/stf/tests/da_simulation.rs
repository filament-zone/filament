use borsh::BorshSerialize;
use filament_hub_stf::runtime::Runtime;
use sov_bank::Bank;
use sov_mock_da::MockDaSpec;
use sov_modules_api::{
    runtime::capabilities::RawTx,
    transaction::Transaction,
    EncodeCall,
    PrivateKey,
};
use sov_test_utils::{bank_data::BankMessageGenerator, MessageGenerator, TestPrivateKey};

pub(crate) type S = sov_test_utils::TestSpec;
type Da = MockDaSpec;

pub fn simulate_da(admin: TestPrivateKey) -> Vec<RawTx> {
    let mut messages = Vec::default();

    let bank_generator = BankMessageGenerator::<S>::with_minter_and_transfer(admin.clone());
    let bank_messages = bank_generator.create_default_messages_without_gas_usage();

    let nonce_offset = messages.len() as u64;
    for mut msg in bank_messages {
        msg.nonce += nonce_offset;
        let tx = msg.to_tx::<Runtime<S, Da>>();
        messages.push(RawTx {
            data: tx.try_to_vec().unwrap(),
        });
    }
    messages
}

pub fn simulate_da_with_revert_msg(admin: TestPrivateKey) -> Vec<RawTx> {
    let mut messages = Vec::default();
    let bank_generator = BankMessageGenerator::<S>::create_invalid_transfer(admin);
    let bank_txns = bank_generator.create_default_raw_txs_without_gas_usage::<Runtime<S, Da>>();
    messages.extend(bank_txns);
    messages
}

pub fn simulate_da_with_bad_sig(key: TestPrivateKey) -> Vec<RawTx> {
    let b: BankMessageGenerator<S> = BankMessageGenerator::with_minter(key.clone());
    let create_token_message = b.create_default_messages().remove(0);
    let tx = Transaction::<S>::new(
        create_token_message.sender_key.pub_key(),
        <Runtime<S, Da> as EncodeCall<Bank<S>>>::encode_call(create_token_message.content.clone()),
        // Use the signature of an empty message
        key.sign(&[]),
        create_token_message.chain_id,
        create_token_message.max_priority_fee_bips,
        create_token_message.max_fee,
        create_token_message.gas_limit,
        create_token_message.nonce,
    );
    // Overwrite the signature with the signature of the empty message

    vec![RawTx {
        data: tx.try_to_vec().unwrap(),
    }]
}

pub fn simulate_da_with_bad_nonce(key: TestPrivateKey) -> Vec<RawTx> {
    let b: BankMessageGenerator<S> = BankMessageGenerator::with_minter(key);
    let mut create_token_message = b.create_default_messages().remove(0);
    // Overwrite the nonce with the maximum value
    create_token_message.nonce = u64::MAX;
    let tx = create_token_message.to_tx::<Runtime<S, Da>>();
    vec![RawTx {
        data: tx.try_to_vec().unwrap(),
    }]
}

pub fn simulate_da_with_bad_serialization(key: TestPrivateKey) -> Vec<RawTx> {
    let b: BankMessageGenerator<S> = BankMessageGenerator::with_minter(key);
    let create_token_message = b.create_default_messages().remove(0);
    let tx = Transaction::<S>::new_signed_tx(
        &create_token_message.sender_key,
        b"not a real call message".to_vec(),
        create_token_message.chain_id,
        create_token_message.max_priority_fee_bips,
        create_token_message.max_fee,
        create_token_message.gas_limit,
        create_token_message.nonce,
    );

    vec![RawTx {
        data: tx.try_to_vec().unwrap(),
    }]
}
