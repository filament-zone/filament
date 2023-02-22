use std::{ffi::OsString, fs};

use pulzaar_app::accounts;
use pulzaar_chain::{
    input::Transfer,
    Account,
    Address,
    Amount,
    Auth,
    ChainId,
    Input,
    Transaction,
    TransactionBody,
};
use pulzaar_crypto::{SignBytes, SigningKey};
use pulzaar_encoding::{from_bytes, to_bytes, FromBech32 as _};
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

    let (amount, denom) = match args.get(3).unwrap().to_owned().into_string() {
        Ok(value) => {
            let n = value
                .find(|c| !char::is_numeric(c))
                .ok_or(eyre::eyre!("asset id missing"))?;
            let (amount, denom) = value.split_at(n);

            (Amount::try_from(amount)?, denom.to_owned())
        },
        Err(e) => return Err(eyre::eyre!("parsing funds failed: {:?}", e)),
    };

    // TODO(tsenart): Make node URI a flag.
    let rt = Runtime::new()?;
    let client = HttpClient::new("http://127.0.0.1:26657")?;
    let path = Some("/accounts".to_string());
    let data = to_bytes(&accounts::Query::AccountByAddress(from.clone()))?;
    let query = client.abci_query(path, data, None, false);
    let res = rt.block_on(query)?;

    if res.code.is_err() {
        eyre::bail!("ABCI account query error {:?}", res);
    }

    let account: Account = from_bytes(&res.value)?;

    let transfer = Transfer {
        from,
        to,
        denom,
        amount,
    };

    let body = TransactionBody {
        inputs: vec![Input::Transfer(transfer)],
        chain_id,
        max_height: None,
        account_id: account.id(),
        sequence: account.sequence(),
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

    let broadcast = client.broadcast_tx_commit::<Vec<u8>>(tx_bytes);
    let res = rt.block_on(broadcast)?;

    println!("{:?}", res);

    Ok(())
}

fn print_usage() {
    println!("\nUsage: {NAME} transfer <chain-id> <from-address> <to-address> <funds>\n");
}
