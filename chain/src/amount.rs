use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
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
