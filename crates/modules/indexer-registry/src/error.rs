use sov_modules_api::Spec;
use thiserror::Error;

#[derive(Debug, Eq, PartialEq, Error)]
pub enum IndexerRegistryError<S: Spec> {
    #[error("Module admin is not set. This is a bug - the admin should be set at genesis")]
    AdminNotSet,

    #[error("Indexer '{indexer}' is not registered")]
    IndexerNotRegistered { indexer: S::Address },

    #[error("Sender '{sender}' is not an admin")]
    SenderNotAdmin { sender: S::Address },
}
