use std::ffi::OsString;

use eyre::eyre;
use filament_node::Config;
use tokio::runtime::Runtime;

use crate::{
    context::Context,
    terminal::{Args, Help},
};

pub const HELP: Help = Help {
    name: "node",
    description: "Run a filament node",
    version: env!("CARGO_PKG_VERSION"),
    usage: r#"
Usage

    flt node

Options

    --host <ip>
    --abci-port <port>
    --metrics-port <port>
    "#,
};

pub type Options = Config;

impl Args for Options {
    fn from_args(args: Vec<OsString>) -> eyre::Result<Self> {
        use lexopt::prelude::*;

        let mut parser = lexopt::Parser::from_args(args);
        let mut cfg = Config::default();

        while let Some(arg) = parser.next()? {
            match arg {
                Long("host") => {
                    cfg.host = parser
                        .value()?
                        .to_str()
                        .ok_or(eyre!("host is not UTF-8"))?
                        .to_owned();
                },
                Long("abci-port") => {
                    cfg.abci_port = parser
                        .value()?
                        .to_str()
                        .ok_or(eyre!("host is not UTF-8"))?
                        .to_owned();
                },
                Long("metrics-port") => {
                    cfg.metrics_port = parser
                        .value()?
                        .to_str()
                        .ok_or(eyre!("host is not UTF-8"))?
                        .to_owned();
                },
                _ => return Err(eyre!(arg.unexpected())),
            }
        }

        todo!()
    }
}

pub fn run(_ctx: Context, cfg: Options) -> eyre::Result<()> {
    let rt = Runtime::new()?;

    rt.block_on(filament_node::run(cfg))
}
