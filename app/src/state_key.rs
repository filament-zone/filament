use filament_chain::{Address, AssetId};

pub trait StateKey {
    fn state_key(&self) -> String;
}

impl StateKey for Address {
    fn state_key(&self) -> String {
        hex::encode(self)
    }
}

impl StateKey for AssetId {
    fn state_key(&self) -> String {
        // FIXME(xla): Conversion here should be safe and respect character set that is tolerated in
        // state keys.
        self.to_string()
    }
}

#[inline]
pub fn block_height() -> &'static str {
    "block_height"
}

#[inline]
pub fn block_timestamp() -> &'static str {
    "block_timestamp"
}

#[inline]
pub fn chain_parameters() -> &'static str {
    "chain_parameters"
}
