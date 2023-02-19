use serde::{Deserialize, Serialize};

use crate::{asset, Amount};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Funds {
    asset_id: asset::Id,
    amount: Amount,
}

impl TryFrom<&str> for Funds {
    type Error = eyre::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let n = value
            .find(|c| !char::is_numeric(c))
            .ok_or(eyre::eyre!("no asset_id found"))?;

        let (amount, asset_id) = value.split_at(n);

        let amount = Amount::try_from(amount)?;
        let asset_id = asset::Id(asset_id.to_string());

        Ok(Funds { asset_id, amount })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn funds_try_from() {
        assert!(Funds::try_from("plz").is_err());
        assert!(Funds::try_from("1000").is_err());

        let amount = Amount::from(1000);
        let asset_id = asset::Id("plz".to_string());
        assert_eq!(
            Funds::try_from("1000plz").unwrap(),
            Funds { amount, asset_id },
        );
    }
}
