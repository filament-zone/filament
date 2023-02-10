use tokio::runtime::Runtime;

use crate::terminal::Args;

impl Args for pulzaard::Config {
    #[allow(clippy::never_loop, clippy::match_single_binding)]
    fn from_args(args: Vec<std::ffi::OsString>) -> eyre::Result<(Self, Vec<std::ffi::OsString>)> {
        let mut parser = lexopt::Parser::from_args(args);
        while let Some(arg) = parser.next()? {
            match arg {
                _ => return Err(eyre::eyre!(arg.unexpected())),
            }
        }

        Ok((Default::default(), vec![]))
    }
}

pub fn run(cfg: pulzaard::Config) -> eyre::Result<()> {
    let rt = Runtime::new()?;

    rt.block_on(pulzaard::run(cfg))
}
