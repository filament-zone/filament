use serde::{Deserialize, Serialize};

mod delegate;
mod undelegate;

pub use delegate::Delegate;
pub use undelegate::Undelegate;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum Input {
    // Staking
    Delegate(Delegate),
    Undelegate(Undelegate),
}
