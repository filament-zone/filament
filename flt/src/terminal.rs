use std::{ffi::OsString, fmt, process};

use dialoguer::console::style;

mod args;
mod error;

pub use args::Args;
pub use error::Error;

pub trait Command<A, Ctx>
where
    A: Args,
{
    fn run(self, ctx: Ctx, args: A) -> eyre::Result<()>;
}

impl<F, A, Ctx> Command<A, Ctx> for F
where
    A: Args,
    F: FnOnce(Ctx, A) -> eyre::Result<()>,
{
    fn run(self, ctx: Ctx, args: A) -> eyre::Result<()> {
        self(ctx, args)
    }
}

pub struct Help {
    pub name: &'static str,
    pub description: &'static str,
    pub version: &'static str,
    pub usage: &'static str,
}

pub fn error(err: impl fmt::Display) {
    println!("{} {}", style("==").red(), style(err).red());
}

pub fn run<A, Ctx, C>(ctx: Ctx, help: Help, cmd: C, args: Vec<OsString>) -> !
where
    A: Args,
    C: Command<A, Ctx>,
{
    let options = match A::from_args(args) {
        Ok(opts) => opts,
        Err(err) => {
            match err.downcast_ref::<Error>() {
                Some(Error::Help) => {
                    println!(
                        "{} {}\n{}\n{}",
                        help.name, help.version, help.description, help.usage
                    );
                    process::exit(0);
                },
                Some(Error::Usage) => {
                    println!(
                        "{}\n{}",
                        style(format!("Error: {} invalid usage", help.name)).red(),
                        style(help.usage).red().dim()
                    );
                    process::exit(1);
                },
                _ => {},
            };

            fail(help.name, &err);
        },
    };

    match cmd.run(ctx, options) {
        Ok(()) => process::exit(0),
        Err(err) => {
            fail(&format!("{} failed", help.name), &err);
        },
    }
}

fn fail(name: &str, err: &eyre::Report) -> ! {
    eprintln!(
        "{} {} {} {}",
        style("==").red(),
        style("Error").red(),
        style(format!("{}:", name)).red(),
        style(err.to_string()).red(),
    );

    if let Some(Error::Hint { hint, .. }) = err.downcast_ref::<Error>() {
        eprintln!("{}", style(hint).yellow());
    }

    process::exit(1);
}
