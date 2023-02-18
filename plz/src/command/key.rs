use std::ffi::OsString;

use pulzaar_crypto::{Address, SigningKey};
use pulzaar_encoding::ToBech32 as _;
use rand::thread_rng;

pub fn generate(_args: &[OsString]) -> eyre::Result<()> {
    let sk = SigningKey::new(thread_rng());
    let address = Address(sk.verification_key());

    let sk_bech32 = sk.to_bech32()?;
    let address_bech32 = address.to_bech32()?;

    println!("address: {address_bech32}\nsigning_key: {sk_bech32}");

    Ok(())
}
