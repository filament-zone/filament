#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use std::assert_eq;

use filament_hub_wasm::{new_serialized_unsigned_tx, serialize_call};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

const RUNTIME_CALL_JSON: &str = r#"{"bank":{"freeze":{"token_id":"token_1rwrh8gn2py0dl4vv65twgctmlwck6esm2as9dftumcw89kqqn3nqrduss6"}}}"#;
const RUNTIME_CALL_SERIALIZED: [u8; 34] = [
    0, 4, 27, 135, 115, 162, 106, 9, 30, 223, 213, 140, 213, 22, 228, 97, 123, 251, 177, 109, 102,
    27, 87, 96, 86, 165, 124, 222, 28, 114, 216, 0, 156, 102,
];

const UNSIGNED_TX_SERIALIZED: [u8; 71] = [
    34, 0, 0, 0, 0, 4, 27, 135, 115, 162, 106, 9, 30, 223, 213, 140, 213, 22, 228, 97, 123, 251,
    177, 109, 102, 27, 87, 96, 86, 165, 124, 222, 28, 114, 216, 0, 156, 102, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

#[wasm_bindgen_test]
fn test_new_serialized_unsigned_tx() {
    let chain_id: u64 = 0;
    let nonce: u64 = 0;
    let max_priority_fee: u64 = 0;
    let max_fee: u64 = 0;

    let utx = new_serialized_unsigned_tx(
        RUNTIME_CALL_SERIALIZED.into(),
        chain_id,
        max_priority_fee,
        max_fee,
        nonce,
    )
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
