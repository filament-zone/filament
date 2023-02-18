use penumbra_storage::RootHash;
use sha2::{Digest as _, Sha256};

static APPHASH_DOMSEP: &str = "PulzaarAppHash";

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct AppHash(pub [u8; 32]);

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
