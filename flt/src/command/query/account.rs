use eyre::{bail, eyre};
use filament_app::accounts;
use filament_chain::{Account, Address};
use filament_encoding::FromBech32;

use crate::{
    command::Context,
    terminal::{Args, Help},
};

pub const HELP: Help = Help {
    name: "account",
    description: "Inspect an on-chain account",
    version: env!("CARGO_PKG_VERSION"),
    usage: r#"
Usage

    flt query account <address>

Options

    --height <height> Height at which to query
    "#,
};

pub struct Options {
    address: Address,
    height: Option<u64>,
}

impl Args for Options {
    fn from_args(args: Vec<std::ffi::OsString>) -> eyre::Result<Self> {
        use lexopt::prelude::*;

        let mut parser = lexopt::Parser::from_args(args);
        let mut address: Option<Address> = None;
        let mut height: Option<u64> = None;

        while let Some(arg) = parser.next()? {
            match arg {
                Long("height") => {
                    let val = parser.value()?;
                    let val = val
                        .to_str()
                        .ok_or(eyre!("height is not UTF-8"))?
                        .parse::<u64>()?;

                    height = Some(val);
                },
                Value(addr) if address.is_none() => {
                    let addr = addr.to_str().ok_or(eyre!("address is not UTF-8"))?;
                    address = Some(Address::from_bech32(addr)?);
                },
                _ => bail!(arg.unexpected()),
            }
        }

        Ok(Self {
            address: address.ok_or(eyre!("address is missing"))?,
            height,
        })
    }
}

pub fn run(ctx: Context, opts: Options) -> eyre::Result<()> {
    let account = ctx
        .client
        .query::<Account>(opts.height, accounts::Query::AccountByAddress(opts.address))?;

    dbg!(&account);

    Ok(())
}

#[cfg(test)]
mod test {
    use filament_chain::Address;
    use filament_crypto::SigningKey;
    use filament_encoding::ToBech32 as _;
    use pretty_assertions::assert_eq;
    use rand::thread_rng;

    use super::Options;
    use crate::terminal::Args as _;

    #[test]
    fn options() -> eyre::Result<()> {
        let addr = Address::from(SigningKey::new(thread_rng()).verification_key());

        let opts = Options::from_args(vec!["--height=69".into(), addr.to_bech32()?.into()])?;

        assert_eq!(opts.address, addr);
        assert_eq!(opts.height, Some(69));

        Ok(())
    }
}
