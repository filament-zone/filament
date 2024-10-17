use std::io::{self, Write};

use anyhow::{anyhow, Result};
use bip32::{DerivationPath, XPrv};
use bip39::{Language, Mnemonic};
use k256::ecdsa::SigningKey;
use sov_mock_zkvm::MockZkVerifier;
use sov_modules_api::{default_spec::DefaultSpec, execution_mode::Native, CryptoSpec, Spec};

type S = DefaultSpec<MockZkVerifier, MockZkVerifier, Native>;

fn derive_signing_key_from_mnemonic(mnemonic: &str) -> Result<SigningKey> {
    // Parse the mnemonic phrase
    let mnemonic = Mnemonic::parse_in_normalized(Language::English, mnemonic)
        .map_err(|e| anyhow!("mnemonic parsing failed: {}", e))?;

    // Convert the mnemonic to a seed with an empty passphrase (MetaMask default)
    let seed = mnemonic.to_seed_normalized("");

    // Define MetaMask's Ethereum derivation path
    let derivation_path = "m/44'/60'/0'/0/0".parse::<DerivationPath>()?;

    // Derive the private key at the specific path
    let xprv = XPrv::derive_from_path(seed, &derivation_path)?;

    // Convert to k256 SigningKey
    let signing_key = SigningKey::from_bytes(&xprv.to_bytes().into())?;

    Ok(signing_key)
}

fn main() -> Result<()> {
    // Prompt the user to enter their mnemonic phrase
    print!("Enter your 12-word mnemonic phrase: ");
    io::stdout().flush()?; // Ensure the prompt appears before input

    // Read the mnemonic phrase from standard input
    let mut mnemonic_phrase = String::new();
    io::stdin().read_line(&mut mnemonic_phrase)?;
    let mnemonic_phrase = mnemonic_phrase.trim(); // Trim any whitespace/newline

    // Derive the private key and print it
    let sk = derive_signing_key_from_mnemonic(mnemonic_phrase)?;
    let addr = filament_hub_eth::vk_to_address::<S>(sk.verifying_key())?;
    let credential_id = filament_hub_eth::vk_to_credential_id::<
        <<S as Spec>::CryptoSpec as CryptoSpec>::Hasher,
    >(sk.verifying_key());

    println!("addr: {}", addr);
    println!("credential_id: {}", credential_id);

    Ok(())
}
