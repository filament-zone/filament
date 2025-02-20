#![allow(unused_imports)]
#![allow(dead_code)]
use crate::database::Database;
use crate::error::Error;
use crate::ethereum::{DelegateSetChangedEvent, EthereumClient};
use crate::hub::HubClient;
use async_trait::async_trait;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Mutex;
use web3::types::{H160, H256}; // Import FromStr

// Mock EthereumClient
#[derive(Debug, Clone, Default)]
pub struct MockEthereumClient {
    pub events: Vec<DelegateSetChangedEvent>,
    pub next_block: u64,
    pub fail_get_latest_block: bool, // Simulate an error
    pub fail_event_retrieval: bool,
}

#[async_trait]
impl EthereumClient for MockEthereumClient {
    type Transport = web3::transports::http::Http;

    async fn get_latest_block_number(&self) -> Result<u64, Error> {
        if self.fail_get_latest_block {
            Err(Error::EthereumRpcError(web3::Error::Unreachable)) // Example error
        } else {
            Ok(self.next_block)
        }
    }

    async fn get_delegate_set_changed_events(
        &self,
        from_block: u64,
        to_block: u64,
    ) -> Result<Vec<DelegateSetChangedEvent>, Error> {
        if self.fail_event_retrieval {
            return Err(Error::EthereumRpcError(web3::Error::Unreachable));
        }
        let mut filtered_events = Vec::new();
        for event in &self.events {
            if event.block_number >= from_block && event.block_number <= to_block {
                filtered_events.push(event.clone());
            }
        }
        Ok(filtered_events)
    }
    fn clone_boxed(&self) -> Box<dyn EthereumClient<Transport = Self::Transport> + Send + Sync> {
        Box::new(self.clone())
    }
}

// Mock HubClient
#[derive(Debug, Clone, Default)]
pub struct MockHubClient {
    pub sent_transactions: Arc<Mutex<Vec<(String, u64)>>>, // Track sent transactions
    pub fail_update_voting_power: bool,                    // Simulate an error
    pub confirmation_delay: Option<u64>, // Simulate confirmation delay (in blocks)
    pub confirmed_transactions: Arc<Mutex<HashMap<String, bool>>>, // Track confirmed transactions
}

#[async_trait]
impl HubClient for MockHubClient {
    async fn update_voting_power(&self, addr: String, power: u64) -> Result<String, Error> {
        if self.fail_update_voting_power {
            Err(Error::HubError(
                "Simulated update_voting_power failure".to_string(),
            ))
        } else {
            let mut sent_txs = self.sent_transactions.lock().unwrap();
            sent_txs.push((addr.clone(), power));
            // Create a mock tx_hash
            let tx_hash = format!("0x{}", hex::encode(addr.as_bytes()));
            Ok(tx_hash)
        }
    }

    async fn get_tx_status(&self, tx_hash: &str) -> Result<Option<serde_json::Value>, Error> {
        // Simulate confirmation status
        let confirmed = self
            .confirmed_transactions
            .lock()
            .unwrap()
            .get(tx_hash)
            .cloned();
        match confirmed {
            Some(true) => Ok(Some(serde_json::json!({"status": "confirmed"}))), // Simplified status
            Some(false) | None => Ok(None), // Simulate pending or not found
        }
    }

    async fn await_transaction_confirmation(
        &self,
        tx_hash: &String,
        _retries: u32,
        _delay_seconds: u64,
    ) -> Result<(), Error> {
        // Simulate a confirmation delay if configured.
        if let Some(delay) = self.confirmation_delay {
            std::thread::sleep(std::time::Duration::from_secs(delay));
        }

        // Check if the transaction is marked as confirmed.
        let confirmed = self
            .confirmed_transactions
            .lock()
            .unwrap()
            .get(tx_hash)
            .cloned();

        match confirmed {
            Some(true) => Ok(()),                   // Transaction confirmed
            _ => Err(Error::HubConfirmationFailed), // Transaction not confirmed or doesn't exist
        }
    }

    fn clone_boxed(&self) -> Box<dyn HubClient + Send + Sync> {
        Box::new(self.clone())
    }
}

// Mock Database.  Uses in-memory HashMap for simplicity.
#[derive(Debug, Clone, Default)]
pub struct MockDatabase {
    pub last_processed_block: Arc<Mutex<Option<u64>>>,
    pub delegate_powers: Arc<Mutex<HashMap<String, u64>>>,
    pub fail_save_block: bool, // Simulate an error
    pub fail_get_block: bool,  // Simulate an error
    pub fail_update_power: bool,
    pub initialized: Arc<Mutex<bool>>,
}
impl MockDatabase {
    pub fn set_confirmed(&mut self, tx_hash: &str) {
        self.confirmed_transactions
            .lock()
            .unwrap()
            .insert(tx_hash.to_string(), true);
    }
}

impl Database for MockDatabase {
    fn initialize(&self, genesis_block: u64) -> Result<(), Error> {
        let mut initialized = self.initialized.lock().unwrap();
        if !*initialized {
            self.save_last_processed_block(genesis_block)?;
            *initialized = true;
        }
        Ok(())
    }

    fn save_last_processed_block(&self, block_number: u64) -> Result<(), Error> {
        if self.fail_save_block {
            Err(Error::DatabaseError(
                "Simulated save_last_processed_block failure".to_string(),
            ))
        } else {
            *self.last_processed_block.lock().unwrap() = Some(block_number);
            Ok(())
        }
    }

    fn get_last_processed_block(&self) -> Result<Option<u64>, Error> {
        if self.fail_get_block {
            Err(Error::DatabaseError(
                "Simulated get_last_processed_block failure".to_string(),
            ))
        } else {
            Ok(*self.last_processed_block.lock().unwrap())
        }
    }

    fn update_delegate_power(&self, delegate: &H160, power: u64) -> Result<(), Error> {
        let address_string = format!("{:?}", delegate);
        if self.fail_update_power {
            Err(Error::DatabaseError(
                "Simulated update_delegate_power failure".to_string(),
            ))
        } else {
            self.delegate_powers
                .lock()
                .unwrap()
                .insert(address_string, power);
            Ok(())
        }
    }

    fn get_delegate_power(&self, delegate: &H160) -> Result<Option<u64>, Error> {
        Ok(self
            .delegate_powers
            .lock()
            .unwrap()
            .get(&format!("{:?}", delegate))
            .cloned())
    }

    fn get_all_delegate_powers(&self) -> Result<HashMap<H160, u64>, Error> {
        let mut map = HashMap::new();
        for (addr_str, power) in self.delegate_powers.lock().unwrap().iter() {
            let addr = H160::from_str(addr_str)
                .map_err(|e| Error::Other(format!("Invalid address format {}", e)))?;
            map.insert(addr, *power);
        }
        Ok(map)
    }

    fn clear_database(&self) -> Result<(), Error> {
        *self.last_processed_block.lock().unwrap() = None;
        self.delegate_powers.lock().unwrap().clear();
        Ok(())
    }
    fn clone_boxed(&self) -> Box<dyn Database + Send + Sync> {
        Box::new(self.clone())
    }
}

// Helper function to create a DelegateSetChangedEvent
pub fn create_delegate_set_changed_event(
    delegates: Vec<String>,
    block_number: u64,
) -> DelegateSetChangedEvent {
    let h160_delegates = delegates
        .iter()
        .map(|d| H160::from_str(d).unwrap())
        .collect();
    DelegateSetChangedEvent {
        delegates: h160_delegates,
        block_number,
        transaction_hash: H256::default(), // You can put a mock hash here if needed
    }
}
