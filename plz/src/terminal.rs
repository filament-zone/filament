use std::fmt;

use dialoguer::console::style;

mod args;

use args::Args;

pub trait Command<A: Args> {
    fn run(self, args: A) -> eyre::Result<()>;
}

pub trait Context {}

pub struct Help {
    pub name: &'static str,
    pub description: &'static str,
    pub version: &'static str,
    pub usage: &'static str,
}

pub fn error(err: impl fmt::Display) {
    println!("{} {}", style("==").red(), style(err).red());
}
