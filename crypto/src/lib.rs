mod address;

pub use address::Address;
pub use ed25519_consensus::{Error as Ed25519Error, Signature, SigningKey, VerificationKey};
