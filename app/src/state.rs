use async_trait::async_trait;
use penumbra_storage::Snapshot;
use pulzaar_encoding::StateWriteEncode;
use tendermint::Time;

use crate::AppHash;

#[async_trait]
pub trait AppHashRead {
    async fn app_hash(&self) -> eyre::Result<AppHash>;
}

#[async_trait]
impl AppHashRead for Snapshot {
    async fn app_hash(&self) -> eyre::Result<AppHash> {
        let root = self.root_hash().await.map_err(|err| eyre::eyre!(err))?;
        Ok(AppHash::from(root))
    }
}

pub fn block_height() -> &'static str {
    "block_height"
}

pub fn block_timestamp() -> &'static str {
    "block_timestamp"
}

#[async_trait]
pub trait StateWriteExt: StateWriteEncode {
    /// Writes the block height to the JMT
    fn put_block_height(&mut self, height: u64) -> eyre::Result<()> {
        self.put_bcs(block_height().into(), &height)
    }

    /// Writes the block timestamp to the JMT
    fn put_block_timestamp(&mut self, timestamp: Time) -> eyre::Result<()> {
        self.put_bcs(block_timestamp().into(), &timestamp.to_rfc3339())
    }
}

impl<T: StateWriteEncode + ?Sized> StateWriteExt for T {}
