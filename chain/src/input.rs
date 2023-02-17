use serde::{Deserialize, Serialize};

mod assets;
mod delegate;
mod undelegate;

pub use assets::Transfer;
pub use delegate::Delegate;
pub use undelegate::Undelegate;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum Input {
    // Assets
    Transfer(Transfer),

    // Staking
    Delegate(Delegate),
    Undelegate(Undelegate),
}
