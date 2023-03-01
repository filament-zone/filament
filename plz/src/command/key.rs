use std::{ffi::OsString, fs};

use pulzaar_chain::Address;
use pulzaar_crypto::SigningKey;
use pulzaar_encoding::ToBech32 as _;
use rand::thread_rng;

use crate::{
    context::Context,
    terminal::{Args, Help},
};

pub const HELP: Help = Help {
    name: "key",
    description: "Manage Pulzaar keys",
    version: env!("CARGO_PKG_VERSION"),
    usage: r#"
Usage

    plz key <command>

Options
    "#,
};

pub struct Options {}

impl Args for Options {
    fn from_args(_args: Vec<OsString>) -> eyre::Result<Self> {
        Ok(Self {})
    }
}

pub fn run(ctx: Context, _opts: Options) -> eyre::Result<()> {
    let config_dir = ctx.dirs.config_dir();

    let sk = SigningKey::new(thread_rng());
    let address = Address::from(sk.verification_key()).to_bech32()?;

    let keys_path = config_dir.join("keys");
    fs::create_dir_all(&keys_path)?;

    let key_path = keys_path.join(&address);
    fs::write(&key_path, sk.to_bytes())?;

    println!("Wrote key for {} to {}", address, key_path.display());

    Ok(())
}
