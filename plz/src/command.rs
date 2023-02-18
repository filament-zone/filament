use std::ffi::OsString;

use crate::config::{DESCRIPTION, GIT_HEAD, NAME, VERSION};

mod key;
mod node;

#[derive(Debug)]
pub enum Root {
    Help,
    Version,

    Cmd(Vec<OsString>),
}

impl Root {
    pub fn run(&self) -> eyre::Result<()> {
        match self {
            Self::Help => print_usage(),
            Self::Version => print_version(),
            Self::Cmd(args) => {
                let cmd = args.first();

                if let Some(Some(cmd)) = cmd.map(|s| s.to_str()) {
                    run_cmd(cmd, &args[1..])
                } else {
                    print_usage()
                }
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

fn run_cmd(cmd: &str, args: &[OsString]) -> eyre::Result<()> {
    match cmd {
        "key" => key::generate(args),
        "node" => node::run(args),
        "version" => print_version(),

        _ => Err(eyre::eyre!("\"{cmd}\" is not a supported command")),
    }
}
