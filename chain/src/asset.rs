// TODO(xla): Document.
pub struct Asset {
    pub id: Id,
    pub denom: Denom,
}

// TODO(xla): Needs to get substantially more sophisticated to support stable and transparent use
// of ibc assets.
pub struct Id(pub String);

/// Denominiation of an asset.
// TODO(xla): Document.
pub struct Denom {
    id: Id,
    base: String,
    units: Vec<Unit>,
}

// TODO(xla): Document.
pub struct Unit {
    pub exponent: u8,
    pub denom: String,
}
