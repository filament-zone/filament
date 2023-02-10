use serde::{Deserialize, Serialize};

mod delegate;
mod undelegate;

use delegate::Delegate;
use undelegate::Undelegate;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum Input {
    // Staking
    Delegate(Delegate),
    Undelegate(Undelegate),
}
