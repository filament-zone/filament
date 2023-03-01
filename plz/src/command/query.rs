use std::ffi::OsString;

use eyre::{bail, eyre, Report};

use crate::{
    command::Context,
    terminal::{self, Args, Help},
};

mod account;

#[derive(Debug)]
enum Command {
    Account,
}

impl TryFrom<&str> for Command {
    type Error = Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "account" => Ok(Self::Account),
            _ => eyre::bail!("unknown command: {}", value),
        }
    }
}

pub const HELP: Help = Help {
    name: "query",
    description: "Query node data, e.g. on-chain state",
    version: env!("CARGO_PKG_VERSION"),
    usage: r#"
Usage

    plz query <command>

Options
    "#,
};

pub struct Options {
    command: Command,
    args: Vec<OsString>,
}

impl Args for Options {
    #[allow(clippy::never_loop)]
    fn from_args(args: Vec<OsString>) -> eyre::Result<Self> {
        use lexopt::prelude::*;

        let mut parser = lexopt::Parser::from_args(args);
        let mut command: Option<Command> = None;

        while let Some(arg) = parser.next()? {
            match arg {
                Value(cmd) if command.is_none() => {
                    let cmd = cmd.to_str().ok_or(eyre!("command is not UTF-8"))?;
                    command = Some(Command::try_from(cmd)?);

                    break;
                },
                _ => bail!(arg.unexpected()),
            }
        }

        Ok(Self {
            command: command.unwrap(),
            args: parser.raw_args()?.collect::<Vec<_>>(),
        })
    }
}

pub fn run(ctx: Context, opts: Options) -> eyre::Result<()> {
    match opts.command {
        Command::Account => terminal::run::<account::Options, Context, _>(
            ctx,
            account::HELP,
            account::run,
            opts.args,
        ),
    }
}
