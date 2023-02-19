use std::{ffi::OsString, fs};

use pulzaar_chain::{
    input::Transfer,
    Address,
    Auth,
    ChainId,
    Funds,
    Input,
    Transaction,
    TransactionBody,
};
use pulzaar_crypto::{SignBytes, SigningKey};
use pulzaar_encoding::{to_bytes, FromBech32};
use tendermint_rpc::{Client as _, HttpClient};
use tokio::runtime::Runtime;

use super::{plz_dirs, NAME};

pub fn run(args: &[OsString]) -> eyre::Result<()> {
    // TODO(tsenart): Better CLI flags parsing, usage, etc.

    if args.len() < 4 {
        print_usage();
        return Err(eyre::eyre!("not enough arguments given"));
    }

    let chain_id = args
        .get(0)
        .unwrap()
        .to_owned()
        .into_string()
        .map_err(|e| eyre::eyre!("{:?}", e))?;

    let chain_id = ChainId::try_from(chain_id)?;

    let signing_key = {
        let from_addr = args
            .get(1)
            .unwrap()
            .to_owned()
            .into_string()
            .map_err(|e| eyre::eyre!("from-address invalid: {e:?}"))?;

        let dirs = plz_dirs()?;
        let config_dir = dirs.config_dir();
        let sk_path = config_dir.join("keys").join(from_addr);
        let sk_bytes = fs::read(sk_path)?;

        SigningKey::try_from(sk_bytes.as_slice())?
    };

    let verification_key = signing_key.verification_key();
    let from = Address::from(verification_key);

    let to = match args.get(2).unwrap().to_owned().into_string() {
        Ok(s) => Address::from_bech32(s)?,
        Err(e) => return Err(eyre::eyre!("to-address invalid: {e:?}")),
    };

    let funds = match args.get(3).unwrap().to_owned().into_string() {
        Ok(s) => Funds::try_from(s.as_str()).map_err(|e| eyre::eyre!(e))?,
        Err(e) => return Err(eyre::eyre!("parsing funds failed: {:?}", e)),
    };

    // TODO(tsenart): Read sequence number and account_number from full node.

    let transfer = Transfer { from, to, funds };
    let body = TransactionBody {
        inputs: vec![Input::Transfer(transfer)],
        chain_id,
        max_height: None,
        account_id: 0,
        sequence: 0,
    };

    let sign_bytes = body.sign_bytes()?;

    let tx = Transaction {
        auth: Auth::Ed25519 {
            verification_key,
            signature: signing_key.sign(&sign_bytes),
        },
        body,
    };
    let tx_bytes = to_bytes(&tx)?;

    // TODO(tsenart): Make node URI a flag.
    let client = HttpClient::new("http://127.0.0.1:26657")?;
    let req = client.broadcast_tx_commit(tx_bytes.into());
    let rt = Runtime::new()?;
    let res = rt.block_on(req)?;

    println!("{:?}", res);

    Ok(())
}

fn print_usage() {
    println!("\nUsage: {NAME} transfer <chain-id> <from-address> <to-address> <funds>\n");
}
