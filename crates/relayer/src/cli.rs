use crate::error::Error;
use crate::relayer::Relayer;
use clap::{Parser, Subcommand};
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Subcommand, Debug)]
pub enum CliCommand {
    /// Starts the relayer
    Start {
        /// Optional: Block number to start syncing from
        #[arg(long)]
        block_number: Option<u64>,
    },
    /// Query the current relayer status.
    Query,
    /// Reset the relayer
    Reset,
}

pub fn run_cli(relayer: Relayer) -> Result<(), Error> {
    let cli = Cli::parse();
    relayer.run_command(cli.command)
}
