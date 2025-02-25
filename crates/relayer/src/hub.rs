#![allow(unused_variables)]
use crate::error::Error;
use async_trait::async_trait;
use std::sync::Arc;
// use base64::{engine::general_purpose, Engine as _};
// use filament_hub_core::CallMessage; // Import the CallMessage enum
//use reqwest::Client;
use serde::{Deserialize, Serialize};
// Struct for the JSON payload to the /sequencer/batches endpoint
#[derive(Serialize, Debug)]
struct SendTxRequest {
    transactions: Vec<String>, // Base64 encoded transactions
}

// Struct to deserialize the response (adapt as needed based on actual Hub response)
#[derive(Deserialize, Debug)]
pub struct SendTxResponse {
    tx_hash: String, // Example - adjust based on the actual response
                     // Add other fields as needed (e.g., success/failure status, error messages)
}

#[async_trait]
pub trait HubClientTrait: Send + Sync {
    async fn update_voting_power(&self, addr: String, power: u64) -> Result<String, Error>;
    async fn get_tx_status(&self, tx_hash: &str) -> Result<Option<serde_json::Value>, Error>;
    async fn await_transaction_confirmation(
        &self,
        tx_hash: &String,
        retries: u32,
        delay_seconds: u64,
    ) -> Result<(), Error>;
}

#[derive(Debug, Clone)]
/// [`HubClient`] is a struct that provides methods for interacting with the Filament Hub.
pub struct HubClient {
    pub client: Arc<reqwest::Client>,
    pub hub_url: String,
    // pub hub_address: Address, // If you need the Hub's address
}

#[async_trait]
impl HubClientTrait for HubClient {
    async fn update_voting_power(
        &self,
        addr: String, // Or your Address type
        power: u64,
    ) -> Result<String, Error> {
        Err(Error::Other("Not implemented".to_string()))
        /*
        // 1. Construct the CallMessage
        let call_message = CallMessage::UpdateVotingPower {
            address: addr.clone().into(), // Convert to the appropriate Address type
            power,
        };

        // 2. Serialize the CallMessage (using, for example, borsh)
        let serialized_tx = borsh::to_vec(&call_message)?; // Or use your preferred serialization

        // 3. Base64 encode the serialized transaction
        let base64_tx = general_purpose::STANDARD.encode(&serialized_tx);

        // 4. Create the request payload
        let request_body = SendTxRequest {
            transactions: vec![base64_tx],
        };
        info!("Sending tx: {:?}", request_body);
        // 5. Send the POST request

        let response = self
            .client
            .post(&format!("{}{}", self.hub_url, "/sequencer/batches")) // Assuming hub_url is stored
            .json(&request_body)
            .send()
            .await?;

        // 6. Check the response status
        if response.status().is_success() {
            // 7. Deserialize the response (adjust based on actual response format)
            let response_data: SendTxResponse = response.json().await?;
            //   Do something with response_data.tx_hash (e.g., log it, use it for polling)
            tracing::info!(%addr, %power, tx_hash = %response_data.tx_hash, "Transaction submitted to Hub");
            // TODO(xla): Await confirmation before returning.
            Ok(response_data.tx_hash)
        } else {
            let err = response.text().await?;
            // 8. Handle errors (log, potentially retry)
            tracing::error!(%addr, %power, "Failed to submit transaction to Hub: {:?}", err);
            Err(Error::HubError(format!(
                "Transaction submission failed: {}",
                err
            ))) // Use a custom error variant
        }
         */
    }

    async fn get_tx_status(&self, tx_hash: &str) -> Result<Option<serde_json::Value>, Error> {
        let url = format!("{}/ledger/txs/{}", self.hub_url, tx_hash);
        let resp = self.client.get(&url).send().await?;

        if resp.status().is_success() {
            let data: serde_json::Value = resp.json().await?;
            Ok(Some(data))
        } else if resp.status() == reqwest::StatusCode::NOT_FOUND {
            Ok(None)
        } else {
            Err(Error::HubError(format!(
                "Failed to get tx status: {}",
                resp.status()
            )))
        }
    }

    // Await confirmation from the hub
    async fn await_transaction_confirmation(
        &self,
        tx_hash: &String,
        retries: u32,
        delay_seconds: u64,
    ) -> Result<(), Error> {
        let mut attempt = 0;
        loop {
            let status = self.get_tx_status(tx_hash).await;
            match status {
                Ok(opt) => match opt {
                    Some(tx) => {
                        // TODO(brapse): Ensure tx is correct shape.
                        // TODO(brapse): Return error in case tx failed in some way.
                        tracing::info!("Transaction Confirmed: {}", tx);
                        return Ok(());
                    },
                    // Tx not found, retry...
                    None => {
                        if attempt < retries {
                            tracing::info!("Transaction not found. Retrying...");
                            tokio::time::sleep(tokio::time::Duration::from_secs(delay_seconds))
                                .await;
                            attempt += 1;
                            continue;
                        } else {
                            return Err(Error::HubError(format!(
                                "Transaction not found after {} attempts",
                                retries
                            )));
                        }
                    },
                },
                Err(e) => return Err(Error::HubError(format!("Error retrieving tx: {}", e))),
            }
        }
    }
}

impl HubClient {
    pub fn new(hub_url: String) -> Self {
        let client = Arc::new(reqwest::Client::new());

        Self { client, hub_url }
    }
}
