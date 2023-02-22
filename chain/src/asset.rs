use std::{collections::BTreeMap, fmt::Display};

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

// TODO(xla): Document.
#[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct Asset {
    pub id: Id,
    pub denom: Denom,
}

// TODO(xla): Needs to get substantially more sophisticated to support stable and transparent use
// of ibc assets.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct Id(String);

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<&str> for Id {
    type Error = eyre::Report;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(Self(value.to_owned()))
    }
}

/// Denominiation of an asset.
// TODO(xla): Document.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct Denom {
    pub id: Id,
    pub base: String,
    pub units: Vec<Unit>,
}

// TODO(xla): Document.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct Unit {
    pub exponent: u8,
    pub denom: String,
}

pub struct Registry {
    assets: BTreeMap<Id, Asset>,
}

impl Registry {
    pub fn by_base_denom(&self, denom: &str) -> Option<&Asset> {
        self.assets
            .iter()
            .find(|(_, asset)| asset.denom.base == *denom)
            .map(|(_, asset)| asset)
    }
}

pub static REGISTRY: Lazy<Registry> = Lazy::new(|| {
    let id = Id::try_from("ugm").unwrap();
    let plz = Asset {
        id: id.clone(),
        denom: Denom {
            id: id.clone(),
            base: "ugm".to_owned(),
            units: vec![
                Unit {
                    exponent: 3,
                    denom: "mgm".to_owned(),
                },
                Unit {
                    exponent: 6,
                    denom: "gm".to_owned(),
                },
            ],
        },
    };

    let mut assets = BTreeMap::new();
    assets.insert(id, plz);

    Registry { assets }
});
