// #![no_std]

extern crate alloc;

pub use alloc::vec::Vec;

use borsh::BorshSerialize;
use filament_hub_stf::runtime::RuntimeCall;
use serde::de::DeserializeOwned;
use sov_mock_da::MockDaSpec;
use sov_mock_zkvm::MockZkVerifier;
use sov_modules_api::{
    default_spec::DefaultSpec,
    execution_mode::Zk,
    transaction::{PriorityFeeBips, TxDetails, UnsignedTransaction},
    Spec,
};
use sov_risc0_adapter::Risc0Verifier;
use sov_rollup_interface::zk::CryptoSpec;
use wasm_bindgen::prelude::*;

pub type ZkSpec = DefaultSpec<Risc0Verifier, MockZkVerifier, Zk>;

pub type CSpec = <ZkSpec as Spec>::CryptoSpec;
pub type PublicKey = <CSpec as CryptoSpec>::PublicKey;
pub type Signature = <CSpec as CryptoSpec>::Signature;
pub type Address = <ZkSpec as Spec>::Address;

pub type Call = RuntimeCall<ZkSpec, MockDaSpec>;
pub type UnsignedTx = UnsignedTransaction<ZkSpec>;
pub type Tx = filament_hub_eth::Tx<ZkSpec>;

pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn new_unsigned_tx(runtime_msg: Vec<u8>, chain_id: u64) -> Result<Vec<u8>, JsError> {
    let unsigned_tx = UnsignedTx::new(runtime_msg, chain_id, PriorityFeeBips::ZERO, 100, 0, None);
    serialize_borsh(&unsigned_tx)
}

#[wasm_bindgen]
pub fn new_serialized_tx(
    signature: Vec<u8>,
    verifying_key: Vec<u8>,
    runtime_msg: Vec<u8>,
    chain_id: u64,
) -> Result<Vec<u8>, JsError> {
    let tx = Tx {
        signature,
        verifying_key,
        runtime_msg,
        nonce: 0,
        details: TxDetails {
            chain_id,
            max_priority_fee_bips: PriorityFeeBips::ZERO,
            max_fee: 100,
            gas_limit: None,
        },
    };

    serialize_borsh(&tx)
}

#[wasm_bindgen]
pub fn serialize_call(json: &str) -> Result<Vec<u8>, JsError> {
    serialize_json::<Call>(json)
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
