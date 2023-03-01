use std::ffi::OsString;

use eyre::eyre;

use crate::{
    config::{DESCRIPTION, GIT_HEAD, NAME, VERSION},
    context::Context,
    terminal::run,
};

mod key;
mod node;
mod query;
mod transfer;

pub struct Root(Context, Option<String>, Vec<OsString>);

impl Root {
    pub fn from_env() -> eyre::Result<Self> {
        use lexopt::prelude::*;

        let mut parser = lexopt::Parser::from_env();
        let mut cometbft_uri = "http://127.0.0.1:26657".to_string();
        let mut command = None;
        let mut args = vec![];

        while let Some(arg) = parser.next()? {
            match arg {
                Long("cometbft-uri") => {
                    cometbft_uri = parser
                        .value()?
                        .to_str()
                        .ok_or(eyre!("CometBFT node URI is not UTF-8"))?
                        .to_owned();
                },
                Value(cmd) if command.is_none() => {
                    command = Some(
                        cmd.to_str()
                            .ok_or(eyre!("command is not UTF-8"))?
                            .to_owned(),
                    );
                    args = parser.raw_args()?.collect::<Vec<_>>();
                },
                _ => return Err(eyre!(arg.unexpected())),
            }
        }

        let ctx = Context::new(cometbft_uri)?;

        Ok(Self(ctx, command, args))
    }

    pub fn run(self) -> eyre::Result<()> {
        let Root(ctx, cmd, args) = self;

        match cmd {
            None => print_usage(),
            Some(cmd) => match cmd.as_str() {
                "help" => print_usage(),
                "key" => run::<key::Options, Context, _>(ctx, key::HELP, key::run, args),
                "node" => run::<node::Options, Context, _>(ctx, node::HELP, node::run, args),
                "query" => run::<query::Options, Context, _>(ctx, query::HELP, query::run, args),
                "transfer" => {
                    run::<transfer::Options, Context, _>(ctx, transfer::HELP, transfer::run, args)
                },
                "version" => print_version(),

                _ => Err(eyre::eyre!("\"{cmd}\" is not a supported command")),
            },
        }
    }
}

fn print_usage() -> eyre::Result<()> {
    println!("{DESCRIPTION}");
    println!();
    println!("Usage: {NAME} <command> [--help]");
    println!();

    Ok(())
}

fn print_version() -> eyre::Result<()> {
    println!("{NAME} {VERSION} ({GIT_HEAD})");

    Ok(())
}
