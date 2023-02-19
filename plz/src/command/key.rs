use std::{ffi::OsString, fs};

use pulzaar_chain::Address;
use pulzaar_crypto::SigningKey;
use pulzaar_encoding::ToBech32 as _;
use rand::thread_rng;

use super::plz_dirs;

pub fn generate(_args: &[OsString]) -> eyre::Result<()> {
    let dirs = plz_dirs()?;
    let config_dir = dirs.config_dir();

    let sk = SigningKey::new(thread_rng());
    let address = Address::from(sk.verification_key()).to_bech32()?;

    let keys_path = config_dir.join("keys");
    fs::create_dir_all(&keys_path)?;

    let key_path = keys_path.join(&address);
    fs::write(&key_path, sk.to_bytes())?;

    println!("Wrote key for {} to {}", address, key_path.display());

    Ok(())
}
