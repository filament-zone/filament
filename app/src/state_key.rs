use pulzaar_crypto::Address;

pub trait StateKey {
    fn state_key(&self) -> String;
}

impl StateKey for Address {
    fn state_key(&self) -> String {
        hex::encode(self)
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
