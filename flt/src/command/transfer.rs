use std::{ffi::OsString, fs};

use eyre::{bail, eyre};
use filament_app::accounts;
use filament_chain::{
    input::Transfer,
    Account,
    Address,
    Amount,
    Auth,
    ChainId,
    Denom,
    Input,
    Transaction,
    TransactionBody,
    REGISTRY,
};
use filament_crypto::{SignBytes, SigningKey};
use filament_encoding::{to_bytes, FromBech32 as _, ToBech32};

use crate::{
    context::Context,
    terminal::{Args, Help},
};

pub const HELP: Help = Help {
    name: "transfer",
    description: "Transfer an asset from one account to another",
    version: env!("CARGO_PKG_VERSION"),
    usage: r#"
Usage

    flt transfer <chain_id> <from> <to> <amount> <denom>

Options
    "#,
};

pub struct Options {
    chain_id: ChainId,
    from: Address,
    to: Address,
    amount: Amount,
    denom: Denom,
}

impl Args for Options {
    fn from_args(args: Vec<OsString>) -> eyre::Result<Self> {
        let mut parser = lexopt::Parser::from_args(args);
        let vals = parser.values()?.collect::<Vec<_>>();

        if vals.len() != 5 {
            bail!("expected 5 arguments got {}", vals.len());
        }

        let mut values = vals.iter();

        Ok(Self {
            chain_id: ChainId::try_from(
                values
                    .next()
                    .unwrap()
                    .to_str()
                    .ok_or(eyre!("chain_id is not UTF-8"))?,
            )?,
            from: Address::from_bech32(
                values
                    .next()
                    .unwrap()
                    .to_str()
                    .ok_or(eyre!("from is not UTF-8"))?,
            )?,
            to: Address::from_bech32(
                values
                    .next()
                    .unwrap()
                    .to_str()
                    .ok_or(eyre!("from is not UTF-8"))?,
            )?,
            amount: Amount::try_from(
                values
                    .next()
                    .unwrap()
                    .to_str()
                    .ok_or(eyre!("amount is not UTF-8"))?,
            )?,
            denom: {
                let denom = values
                    .next()
                    .unwrap()
                    .to_str()
                    .ok_or(eyre!("denom is not UTF-8"))?
                    .to_owned();
                let asset = REGISTRY
                    .by_base_denom(&denom)
                    .ok_or(eyre::eyre!("asset not found: {}", denom))?;

                asset.denom.clone()
            },
        })
    }
}

pub fn run(ctx: Context, opts: Options) -> eyre::Result<()> {
    let signing_key = {
        let config_dir = ctx.dirs.config_dir();
        let sk_path = config_dir.join("keys").join(opts.from.to_bech32()?);
        let sk_bytes = fs::read(sk_path)?;

        SigningKey::try_from(sk_bytes.as_slice())?
    };

    let verification_key = signing_key.verification_key();

    let account = ctx
        .client
        .query::<Account>(None, accounts::Query::AccountByAddress(opts.from.clone()))?;

    let transfer = Transfer {
        from: opts.from,
        to: opts.to,
        denom: opts.denom,
        amount: opts.amount,
    };

    let body = TransactionBody {
        inputs: vec![Input::Transfer(transfer)],
        chain_id: opts.chain_id,
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
    let res = ctx.client.broadcast_tx_commit(tx_bytes)?;

    println!("{:?}", res);

    Ok(())
}

#[cfg(test)]
mod test {
    use filament_chain::{Address, Amount, ChainId, REGISTRY};
    use filament_crypto::SigningKey;
    use filament_encoding::ToBech32 as _;
    use pretty_assertions::assert_eq;
    use rand::thread_rng;

    use super::Options;
    use crate::terminal::Args as _;

    #[test]
    fn options() -> eyre::Result<()> {
        let chain_id = ChainId::try_from("inprocess-devnet")?;
        let from = Address::from(SigningKey::new(thread_rng()).verification_key());
        let to = Address::from(SigningKey::new(thread_rng()).verification_key());
        let amount = Amount::try_from("1000")?;
        let asset = REGISTRY.by_base_denom("ugm").unwrap();

        let opts = Options::from_args(vec![
            "inprocess-devnet".into(),
            from.to_bech32()?.into(),
            to.to_bech32()?.into(),
            "1000".into(),
            "ugm".into(),
        ])?;

        assert_eq!(opts.chain_id, chain_id);
        assert_eq!(opts.from, from);
        assert_eq!(opts.to, to);
        assert_eq!(opts.amount, amount);
        assert_eq!(opts.denom, asset.denom);

        Ok(())
    }
}
