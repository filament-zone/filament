use std::ffi::OsString;

use tokio::runtime::Runtime;

// TODO(xla): Parse args into pulzaar-node::Config.
pub fn run(_args: &[OsString]) -> eyre::Result<()> {
    let rt = Runtime::new()?;

    rt.block_on(pulzaar_node::run(Default::default()))
}
