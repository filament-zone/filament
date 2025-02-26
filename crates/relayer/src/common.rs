#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_variables)]

use crate::database::DatabaseTrait;
use crate::error::Error;
use crate::ethereum::{DelegateSetChangedEvent, EthereumClientTrait}; // Import the *trait*
use crate::hub::HubClient;
use crate::hub::HubClientTrait;
use async_trait::async_trait;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use web3::types::{H160, H256};

pub fn create_delegate_set_changed_event(
    delegate_addresses: Vec<String>,
    block_number: u64,
) -> DelegateSetChangedEvent {
    let delegates = delegate_addresses
        .into_iter()
        .map(|addr| H160::from_str(&addr).unwrap())
        .collect();

    DelegateSetChangedEvent {
        delegates,
        block_number,
        transaction_hash: H256::zero(), // For testing purposes, we can use a zero hash
    }
}
#[derive(Clone, Default)]
pub struct MockDatabase {
    last_processed_block: Arc<Mutex<Option<u64>>>,
    delegate_powers: Arc<Mutex<HashMap<H160, u64>>>,
}

impl MockDatabase {
    pub fn new() -> Self {
        Self::default()
    }
}

impl DatabaseTrait for MockDatabase {
    fn initialize(&self, genesis_block: u64) -> Result<(), Error> {
        let mut last_block = self.last_processed_block.lock().unwrap();
        if last_block.is_none() {
            *last_block = Some(genesis_block);
        }
        Ok(())
    }

    fn save_last_processed_block(&self, block_number: u64) -> Result<(), Error> {
        let mut last_block = self.last_processed_block.lock().unwrap();
        *last_block = Some(block_number);
        Ok(())
    }

    fn get_last_processed_block(&self) -> Result<Option<u64>, Error> {
        Ok(*self.last_processed_block.lock().unwrap())
    }

    fn update_delegate_power(&self, delegate: &H160, power: u64) -> Result<(), Error> {
        let mut powers = self.delegate_powers.lock().unwrap();
        powers.insert(*delegate, power);
        Ok(())
    }

    fn get_delegate_power(&self, delegate: &H160) -> Result<Option<u64>, Error> {
        let powers = self.delegate_powers.lock().unwrap();
        Ok(powers.get(delegate).copied())
    }

    fn get_all_delegate_powers(&self) -> Result<HashMap<H160, u64>, Error> {
        Ok(self.delegate_powers.lock().unwrap().clone())
    }

    fn clear_database(&self) -> Result<(), Error> {
        let mut last_block = self.last_processed_block.lock().unwrap();
        *last_block = None;
        let mut powers = self.delegate_powers.lock().unwrap();
        powers.clear();
        Ok(())
    }
}

// Mock EthereumClient
#[derive(Debug, Clone, Default)]
pub struct MockEthereumClient {
    pub events: Vec<DelegateSetChangedEvent>,
    pub next_block: u64,
    pub fail_get_latest_block: bool,
    pub fail_event_retrieval: bool,
}

#[async_trait]
impl EthereumClientTrait for MockEthereumClient {
    // Implement the *trait*
    // ... (rest of your MockEthereumClient methods) ...
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

    async fn get_all_logs(
        &self,
        from_block: web3::types::U64,
        to_block: web3::types::U64,
    ) -> Result<Vec<web3::types::Log>, Error> {
        todo!() // You will add logs if needed for the mock
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
impl HubClientTrait for MockHubClient {
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
}
impl MockHubClient {
    pub fn set_confirmed(&mut self, tx_hash: &str) {
        self.confirmed_transactions
            .lock()
            .unwrap()
            .insert(tx_hash.to_string(), true);
    }
}
