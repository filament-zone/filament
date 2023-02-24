use serde::{Deserialize, Serialize};

mod delegate;
mod transfer;
mod undelegate;

pub use delegate::Delegate;
pub use transfer::Transfer;
pub use undelegate::Undelegate;

// TODO(xla): Remov this allow, it's only triggered because the other variants carry empty structs
// at the moment
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum Input {
    // Assets
    Transfer(Transfer),

    // Staking
    Delegate(Delegate),
    Undelegate(Undelegate),
}
