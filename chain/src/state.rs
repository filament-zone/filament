use async_trait::async_trait;
use penumbra_storage::StateWrite;
use pulzaar_encoding::StateWriteBcs as _;
use tendermint::Time;

pub fn block_height() -> &'static str {
    "block_height"
}

pub fn block_timestamp() -> &'static str {
    "block_timestamp"
}

#[async_trait]
pub trait StateWriteExt: StateWrite {
    /// Writes the block height to the JMT
    fn put_block_height(&mut self, height: u64) -> eyre::Result<()> {
        self.put_bcs(block_height().into(), &height)
    }

    /// Writes the block timestamp to the JMT
    fn put_block_timestamp(&mut self, timestamp: Time) -> eyre::Result<()> {
        self.put_bcs(block_timestamp().into(), &timestamp.to_rfc3339())
    }
}

impl<T: StateWrite + ?Sized> StateWriteExt for T {}
