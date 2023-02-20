use std::{ffi::OsString, fs};

use pulzaar_chain::Address;
use pulzaar_crypto::SigningKey;
use pulzaar_encoding::ToBech32 as _;
use rand::thread_rng;

pub fn generate(_args: &[OsString]) -> eyre::Result<()> {
    let sk = SigningKey::new(thread_rng());

    fs::write("./signing_key", sk.to_bytes())?;

    println!(
        "address: {}",
        Address::from(sk.verification_key()).to_bech32()?
    );

    Ok(())
}
