use std::{iter, process};

use plz::{command::Root, config::NAME, terminal};

fn main() {
    match parse_args().and_then(run) {
        Ok(_) => process::exit(0),
        Err(err) => {
            terminal::error(format!("Error: {NAME}: {err}"));
            process::exit(1);
        },
    }
}

fn parse_args() -> eyre::Result<Root> {
    use lexopt::prelude::*;

    let mut parser = lexopt::Parser::from_env();
    let mut root = None;

    while let Some(arg) = parser.next()? {
        match arg {
            Long("help") | Short('h') => {
                root = Some(Root::Help);
            },
            Long("version") => root = Some(Root::Version),
            Value(cmd) if root.is_none() => {
                let args = iter::once(cmd)
                    .chain(iter::from_fn(|| parser.value().ok()))
                    .collect();

                root = Some(Root::Cmd(args))
            },
            _ => return Err(eyre::eyre!(arg.unexpected())),
        }
    }

    Ok(root.unwrap_or_else(|| Root::Cmd(vec![])))
}

fn run(root: Root) -> eyre::Result<()> {
    root.run()
}
