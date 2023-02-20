use serde::{Deserialize, Serialize};

// TODO(xla): Document.
#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Asset {
    pub id: Id,
    pub denom: Denom,
}

// TODO(xla): Needs to get substantially more sophisticated to support stable and transparent use
// of ibc assets.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Id(pub String);

/// Denominiation of an asset.
// TODO(xla): Document.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Denom {
    id: Id,
    base: String,
    units: Vec<Unit>,
}

// TODO(xla): Document.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Unit {
    pub exponent: u8,
    pub denom: String,
}
