mod delegate;
mod undelegate;

use delegate::Delegate;
use undelegate::Undelegate;

pub enum Input {
    // Staking
    Delegate(Delegate),
    Undelegate(Undelegate),
}
