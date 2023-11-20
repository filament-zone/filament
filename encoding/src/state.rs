use std::{future::Future, pin::Pin};

use eyre::{eyre, WrapErr as _};
use futures::future::{FutureExt as _, TryFutureExt as _};
use penumbra_storage::{StateRead, StateWrite};
use serde::{de::DeserializeOwned, Serialize};

use crate::{from_bytes, to_bytes};

pub trait StateReadDecode: StateRead + Send + Sync {
    fn get_bcs<'a, T>(
        &self,
        key: &'a str,
    ) -> Pin<Box<dyn Future<Output = eyre::Result<Option<T>>> + Send + 'a>>
    where
        T: DeserializeOwned,
    {
        self.get_raw(key)
            .map_err(move |err| eyre!("failed to get raw bytes for {key}: {err}"))
            .and_then(|maybe_bytes| async move {
                match maybe_bytes {
                    None => Ok(None),
                    Some(bytes) => {
                        let v =
                            from_bytes::<T>(&bytes).wrap_err("failed to decode bcs from bytes")?;
                        Ok(Some(v))
                    },
                }
            })
            .boxed()
    }
}

impl<T: StateRead + ?Sized> StateReadDecode for T {}

pub trait StateWriteEncode: StateWrite + Send + Sync {
    fn put_bcs<T>(&mut self, key: String, value: &T) -> eyre::Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.put_raw(key, to_bytes(value)?);

        Ok(())
    }
}

impl<T: StateWrite + ?Sized> StateWriteEncode for T {}

#[cfg(test)]
mod test {
    use penumbra_storage::{StateDelta, Storage};
    use pretty_assertions::assert_eq;
    use serde::{Deserialize, Serialize};
    use tempfile::tempdir;

    use super::{StateReadDecode as _, StateWriteEncode as _};

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
            let storage = Storage::load(path.clone(), vec![])
                .await
                .map_err(|e| eyre::eyre!(e))?;
            let mut state = StateDelta::new(storage.latest_snapshot());
            let mut state_tx = StateDelta::new(&mut state);

            state_tx.put_bcs(key.to_string(), &obj)?;
            state_tx.apply();

            storage.commit(state).await.map_err(|e| eyre::eyre!(e))?;
        }

        // Retrieve value from storage.
        let storage = Storage::load(path, vec![])
            .await
            .map_err(|e| eyre::eyre!(e))?;
        let snapshot = storage.latest_snapshot();

        let v: Object = snapshot.get_bcs(key).await?.unwrap();

        assert_eq!(v, obj);

        Ok(())
    }
}
