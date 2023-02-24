use std::ops;

use num_traits::ops::checked::{CheckedAdd, CheckedSub};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct Amount(u128);

impl From<u128> for Amount {
    fn from(value: u128) -> Self {
        Self(value)
    }
}

impl TryFrom<&str> for Amount {
    type Error = eyre::ErrReport;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let amount = value.parse::<u128>()?;
        Ok(Amount(amount))
    }
}

impl ops::Add<Amount> for Amount {
    type Output = Amount;

    fn add(self, rhs: Amount) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl CheckedAdd for Amount {
    fn checked_add(&self, rhs: &Self) -> Option<Self> {
        self.0.checked_add(rhs.0).map(Self)
    }
}

impl ops::Sub<Amount> for Amount {
    type Output = Amount;

    fn sub(self, rhs: Amount) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl CheckedSub for Amount {
    fn checked_sub(&self, rhs: &Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(Self)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn amount_try_from() {
        assert!(Amount::try_from("not-a-number").is_err());
        assert_eq!(Amount::try_from("1000").unwrap(), Amount(1000));
    }
}
