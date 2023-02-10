use std::{future::Future, pin::Pin};

use eyre::{eyre, WrapErr as _};
use futures::future::{FutureExt as _, TryFutureExt as _};
use penumbra_storage::{StateRead, StateWrite};
use serde::{de::DeserializeOwned, Serialize};

pub trait StateReadBcs: StateRead + Send + Sync {
    fn get_bcs<'a, T>(
        &self,
        key: &'a str,
    ) -> Pin<Box<dyn Future<Output = eyre::Result<Option<T>>> + 'a>>
    where
        T: DeserializeOwned,
    {
        self.get_raw(key)
            .map_err(move |err| eyre!("failed to get raw bytes for {key}: {err}"))
            .and_then(|maybe_bytes| async move {
                match maybe_bytes {
                    None => Ok(None),
                    Some(bytes) => {
                        let v = bcs::from_bytes::<T>(&bytes)
                            .wrap_err("failed to decode bcs from bytes")?;
                        Ok(Some(v))
                    },
                }
            })
            .boxed()
    }
}

impl<T: StateRead + ?Sized> StateReadBcs for T {}

pub trait StateWriteBcs: StateWrite + Send + Sync {
    fn put_bcs<T>(&mut self, key: String, value: &T) -> eyre::Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.put_raw(key, bcs::to_bytes(value)?);

        Ok(())
    }
}

impl<T: StateWrite + ?Sized> StateWriteBcs for T {}

#[cfg(test)]
mod test {
    use penumbra_storage::Storage;
    use pretty_assertions::assert_eq;
    use serde::{Deserialize, Serialize};
    use tempfile::tempdir;

    use super::{StateReadBcs as _, StateWriteBcs as _};

    #[derive(Debug, PartialEq, Deserialize, Serialize)]
    struct Object {
        a: u64,
        b: String,
        c: Vec<u32>,
    }

    #[tokio::test]
    async fn store_and_retrieve() -> eyre::Result<()> {
        let dir = tempdir()?;
        let path = dir.into_path();

        let key = "foo";
        let obj = Object {
            a: rand::random(),
            b: "a/b/c".to_owned(),
            c: vec![rand::random(), rand::random(), rand::random()],
        };

        // Write and commit a value to storage.
        {
            let storage = Storage::load(path.clone())
                .await
                .map_err(|e| eyre::eyre!(e))?;
            let mut state = storage.latest_state();
            let mut state_tx = state.begin_transaction();

            state_tx.put_bcs(key.to_string(), &obj)?;
            state_tx.apply();

            storage.commit(state).await.map_err(|e| eyre::eyre!(e))?;
        }

        // Retrieve value from storage.
        let storage = Storage::load(path).await.map_err(|e| eyre::eyre!(e))?;
        let state = storage.latest_state();

        let v: Object = state.get_bcs(key).await?.unwrap();

        assert_eq!(v, obj);

        Ok(())
    }
}
