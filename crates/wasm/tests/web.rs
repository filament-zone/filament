#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use std::assert_eq;

use filament_hub_wasm::{new_unsigned_tx, serialize_call, tx_hash};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

const RUNTIME_CALL_JSON: &str = r#"{"bank":{"freeze":{"token_id":"token_1rwrh8gn2py0dl4vv65twgctmlwck6esm2as9dftumcw89kqqn3nqrduss6"}}}"#;
const RUNTIME_CALL_SERIALIZED: [u8; 34] = [
    2, 4, 27, 135, 115, 162, 106, 9, 30, 223, 213, 140, 213, 22, 228, 97, 123, 251, 177, 109, 102,
    27, 87, 96, 86, 165, 124, 222, 28, 114, 216, 0, 156, 102,
];

const UNSIGNED_TX_SERIALIZED: [u8; 71] = [
    34, 0, 0, 0, 2, 4, 27, 135, 115, 162, 106, 9, 30, 223, 213, 140, 213, 22, 228, 97, 123, 251,
    177, 109, 102, 27, 87, 96, 86, 165, 124, 222, 28, 114, 216, 0, 156, 102, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

#[wasm_bindgen_test]
fn test_new_serialized_unsigned_tx() {
    let chain_id: u64 = 0;

    let utx = new_unsigned_tx(RUNTIME_CALL_SERIALIZED.into(), chain_id, 0)
        .map_err(JsValue::from)
        .unwrap();
    assert_eq!(utx, UNSIGNED_TX_SERIALIZED);
}

#[wasm_bindgen_test]
fn test_serialize_call() {
    let call = serialize_call(RUNTIME_CALL_JSON)
        .map_err(JsValue::from)
        .unwrap();
    assert_eq!(call, RUNTIME_CALL_SERIALIZED);
}

#[wasm_bindgen_test]
fn test_tx_hash() {
    let tx: [u8; 64] = [0u8; 64];
    assert_eq!(
        tx_hash(tx.to_vec()),
        "0xf5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b".to_string()
    )
}
