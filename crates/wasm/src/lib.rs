// #![no_std]

extern crate alloc;

pub use alloc::vec::Vec;

use borsh::{BorshDeserialize, BorshSerialize};
use filament_hub_stf::runtime::RuntimeCall;
use serde::de::DeserializeOwned;
use sov_mock_da::MockDaSpec;
use sov_mock_zkvm::MockZkVerifier;
use sov_modules_api::{
    default_spec::DefaultSpec,
    execution_mode::Zk,
    transaction::{PriorityFeeBips, Transaction, TxDetails, UnsignedTransaction},
    Spec,
};
use sov_risc0_adapter::{
    crypto::{Risc0PublicKey, Risc0Signature},
    Risc0Verifier,
};
use sov_rollup_interface::zk::CryptoSpec;
use wasm_bindgen::prelude::*;

pub type ZkSpec = DefaultSpec<Risc0Verifier, MockZkVerifier, Zk>;

pub type CSpec = <ZkSpec as Spec>::CryptoSpec;
pub type PublicKey = <CSpec as CryptoSpec>::PublicKey;
pub type Signature = <CSpec as CryptoSpec>::Signature;
pub type Address = <ZkSpec as Spec>::Address;

pub type Call = RuntimeCall<ZkSpec, MockDaSpec>;
pub type UnsignedTx = UnsignedTransaction<ZkSpec>;
pub type Tx = Transaction<ZkSpec>;

pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn new_serialized_unsigned_tx(
    runtime_msg: Vec<u8>,
    chain_id: u64,
    max_priority_fee: u64,
    max_fee: u64,
    nonce: u64,
) -> Result<Vec<u8>, JsError> {
    let unsigned_tx = UnsignedTx::new(
        runtime_msg,
        chain_id,
        max_priority_fee.into(),
        max_fee,
        nonce,
        None,
    );

    serialize_borsh(&unsigned_tx)
}

#[wasm_bindgen]
pub fn new_serialized_tx(
    pub_key: Vec<u8>,
    signature: Vec<u8>,
    message: Vec<u8>,
    chain_id: u64,
    max_priority_fee: u64,
    max_fee: u64,
    nonce: u64,
) -> Result<Vec<u8>, JsError> {
    let pub_key = Risc0PublicKey::try_from_slice(pub_key.as_slice()).map_err(JsError::from)?;
    let signature = Risc0Signature::try_from_slice(signature.as_slice()).map_err(JsError::from)?;

    let tx = Tx::new_with_details(
        pub_key,
        message,
        signature,
        nonce,
        TxDetails {
            chain_id,
            max_priority_fee_bips: PriorityFeeBips::from(max_priority_fee),
            max_fee,
            gas_limit: None,
        },
    );

    serialize_borsh(&tx)
}

#[wasm_bindgen]
pub fn serialize_call(json: &str) -> Result<Vec<u8>, JsError> {
    serialize_json::<Call>(json)
}

#[wasm_bindgen]
pub fn serialize_unsigned_transaction(json: &str) -> Result<Vec<u8>, JsError> {
    serialize_json::<UnsignedTx>(json)
}

fn serialize_borsh<T: BorshSerialize>(obj: &T) -> Result<Vec<u8>, JsError> {
    let mut bytes: Vec<u8> = Vec::new();
    BorshSerialize::serialize(&obj, &mut bytes)
        .map_err(|_| JsError::new("Borsh serialization failed"))?;

    Ok(bytes)
}

fn serialize_json<T>(json: &str) -> Result<Vec<u8>, JsError>
where
    T: BorshSerialize + DeserializeOwned,
{
    let obj: T = serde_json::from_str(json).map_err(JsError::from)?;
    serialize_borsh(&obj)
}
