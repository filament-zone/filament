use crate::error::Error;
use sled::Db;
use std::collections::HashMap;
use web3::types::H160;

const LAST_PROCESSED_BLOCK_KEY: &str = "last_processed_block";

#[derive(Clone)]
pub struct Database {
    db: Db,
}

impl Database {
    pub fn new(path: &str) -> Result<Self, Error> {
        let db = sled::open(path)?;
        Ok(Self { db })
    }

    // Initialize the database, set the genesis block if not already set.
    pub fn initialize(&self, genesis_block: u64) -> Result<(), Error> {
        match self.get_last_processed_block()? {
            Some(_) => Ok(()), // Already initialized
            None => {
                // No value set, initialize with genesis block.
                self.save_last_processed_block(genesis_block)?;
                Ok(())
            },
        }
    }

    pub fn save_last_processed_block(&self, block_number: u64) -> Result<(), Error> {
        let value = bincode::serialize(&block_number)?;
        self.db.insert(LAST_PROCESSED_BLOCK_KEY, value.as_slice())?;
        self.db.flush()?; // Ensure it's written to disk
        Ok(())
    }

    pub fn get_last_processed_block(&self) -> Result<Option<u64>, Error> {
        match self.db.get(LAST_PROCESSED_BLOCK_KEY)? {
            Some(ivec) => {
                let decoded: u64 = bincode::deserialize(&ivec)?;
                Ok(Some(decoded))
            },
            None => Ok(None),
        }
    }

    pub fn update_delegate_power(&self, delegate: &H160, power: u64) -> Result<(), Error> {
        // Serialize the address as bytes
        let key = delegate.as_bytes();
        let value = bincode::serialize(&power)?;
        self.db.insert(key, value.as_slice())?;
        self.db.flush()?;
        Ok(())
    }

    pub fn get_delegate_power(&self, delegate: &H160) -> Result<Option<u64>, Error> {
        // Serialize the address as bytes
        let key = delegate.as_bytes();
        match self.db.get(key)? {
            Some(ivec) => {
                let decoded: u64 = bincode::deserialize(&ivec)?;
                Ok(Some(decoded))
            },
            None => Ok(None),
        }
    }

    pub fn get_all_delegate_powers(&self) -> Result<HashMap<H160, u64>, Error> {
        let mut powers = HashMap::new();
        for result in self.db.iter() {
            let (key, value) = result?;
            // Skip the last processed block key
            if key.as_ref() == LAST_PROCESSED_BLOCK_KEY.as_bytes() {
                continue;
            }

            // Attempt to deserialize the address and power
            let address = H160::from_slice(key.as_ref());
            let power: u64 = bincode::deserialize(&value)?;
            powers.insert(address, power);
        }
        Ok(powers)
    }

    pub fn clear_database(&self) -> Result<(), Error> {
        self.db.clear()?;
        self.db.flush()?;
        Ok(())
    }
}
