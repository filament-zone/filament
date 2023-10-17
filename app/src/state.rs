use async_trait::async_trait;
use filament_chain::ChainParameters;
use filament_encoding::{StateReadDecode, StateWriteEncode};
use tendermint::Time;

use crate::state_key;

#[async_trait]
pub trait StateReadExt: StateReadDecode {
    async fn get_block_height(&self) -> eyre::Result<Option<u64>> {
        self.get_bcs::<u64>(state_key::block_height()).await
    }

    async fn get_block_timestamp(&self) -> eyre::Result<Option<Time>> {
        self.get_bcs::<Time>(state_key::block_timestamp()).await
    }

    async fn get_chain_parameters(&self) -> eyre::Result<Option<ChainParameters>> {
        self.get_bcs::<ChainParameters>(state_key::chain_parameters())
            .await
    }

    async fn get_current_height(&self) -> eyre::Result<u64> {
        Ok(self.get_block_height().await?.unwrap_or_default() + 1)
    }
}

impl<T: StateReadDecode + ?Sized> StateReadExt for T {}

pub trait StateWriteExt: StateWriteEncode {
    /// Writes the block height to the store.
    fn put_block_height(&mut self, height: u64) -> eyre::Result<()> {
        self.put_bcs(state_key::block_height().into(), &height)
    }

    /// Writes the block timestamp to the state.
    fn put_block_timestamp(&mut self, timestamp: Time) -> eyre::Result<()> {
        self.put_bcs(state_key::block_timestamp().into(), &timestamp.to_rfc3339())
    }

    /// Writes the chain parameters to the state.
    fn put_chain_parameters(&mut self, params: &ChainParameters) -> eyre::Result<()> {
        self.put_bcs(state_key::chain_parameters().into(), params)
    }
}

impl<T: StateWriteEncode + ?Sized> StateWriteExt for T {}

#[cfg(test)]
mod test {
    use filament_chain::{ChainId, ChainParameters};
    use penumbra_storage::{StateDelta, Storage};
    use pretty_assertions::assert_eq;
    use tempfile::tempdir;
    use tendermint::Time;

    use super::{StateReadExt as _, StateWriteExt as _};

    #[tokio::test]
    async fn get_current_height() -> eyre::Result<()> {
        let dir = tempdir()?;
        let path = dir.into_path();
        let storage = Storage::load(path.clone())
            .await
            .map_err(|e| eyre::eyre!(e))?;

        {
            let mut state = StateDelta::new(storage.latest_snapshot());
            let mut state_tx = StateDelta::new(&mut state);

            state_tx.put_block_height(123)?;

            state_tx.apply();
            storage.commit(state).await.unwrap();
        }

        // Current height should be one up from the last height stored in the state.
        let state = StateDelta::new(storage.latest_snapshot());
        let current_height = state.get_current_height().await?;

        assert_eq!(current_height, 124);

        Ok(())
    }

    #[tokio::test]
    async fn put_block_height() -> eyre::Result<()> {
        let dir = tempdir()?;
        let path = dir.into_path();
        let storage = Storage::load(path.clone())
            .await
            .map_err(|e| eyre::eyre!(e))?;

        {
            let mut state = StateDelta::new(storage.latest_snapshot());
            let mut state_tx = StateDelta::new(&mut state);

            state_tx.put_block_height(123)?;

            state_tx.apply();
            storage.commit(state).await.unwrap();
        }

        // Height should be present in the state.
        let state = StateDelta::new(storage.latest_snapshot());
        let height = state.get_block_height().await?.unwrap();

        assert_eq!(height, 123);

        Ok(())
    }

    #[tokio::test]
    async fn put_block_timestamp() -> eyre::Result<()> {
        let dir = tempdir()?;
        let path = dir.into_path();
        let storage = Storage::load(path.clone())
            .await
            .map_err(|e| eyre::eyre!(e))?;

        let epoch = Time::unix_epoch();

        {
            let mut state = StateDelta::new(storage.latest_snapshot());
            let mut state_tx = StateDelta::new(&mut state);

            state_tx.put_block_timestamp(epoch)?;

            state_tx.apply();
            storage.commit(state).await.unwrap();
        }

        // Timestamp should be present in the state.
        let state = StateDelta::new(storage.latest_snapshot());
        let stored = state.get_block_timestamp().await?.unwrap();

        assert_eq!(stored, epoch);

        Ok(())
    }

    #[tokio::test]
    async fn put_chain_parameters() -> eyre::Result<()> {
        let dir = tempdir()?;
        let path = dir.into_path();
        let storage = Storage::load(path.clone())
            .await
            .map_err(|e| eyre::eyre!(e))?;

        let params = ChainParameters {
            chain_id: ChainId::try_from("test".to_string())?,
            epoch_duration: 123,
        };

        {
            let mut state = StateDelta::new(storage.latest_snapshot());
            let mut state_tx = StateDelta::new(&mut state);

            state_tx.put_chain_parameters(&params)?;

            state_tx.apply();
            storage.commit(state).await.unwrap();
        }

        // Height should be present in the state.
        let state = StateDelta::new(storage.latest_snapshot());
        let stored = state.get_chain_parameters().await?.unwrap();

        assert_eq!(stored, params);

        Ok(())
    }
}
