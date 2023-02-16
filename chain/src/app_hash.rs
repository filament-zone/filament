use async_trait::async_trait;
use penumbra_storage::{RootHash, Snapshot};
use sha2::{Digest as _, Sha256};

static APPHASH_DOMSEP: &str = "PulzaarAppHash";

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct AppHash(pub [u8; 32]);

#[async_trait]
pub trait AppHashRead {
    async fn app_hash(&self) -> eyre::Result<AppHash>;
}

impl From<RootHash> for AppHash {
    fn from(value: RootHash) -> Self {
        let mut h = Sha256::new();
        h.update(APPHASH_DOMSEP);
        h.update(value.0);

        AppHash(h.finalize().into())
    }
}

impl std::fmt::Debug for AppHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("AppHash")
            .field(&hex::encode(self.0))
            .finish()
    }
}

#[async_trait]
impl AppHashRead for Snapshot {
    async fn app_hash(&self) -> eyre::Result<AppHash> {
        let root = self.root_hash().await.map_err(|err| eyre::eyre!(err))?;
        Ok(AppHash::from(root))
    }
}
