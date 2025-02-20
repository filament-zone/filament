use crate::error::Error;
use std::str::FromStr;
use web3::types::{BlockNumber, FilterBuilder, Log, H160, H256, U256, U64};
use web3::{Transport, Web3};

#[derive(Debug, Clone)]
pub struct DelegateSetChangedEvent {
    pub delegates: Vec<H160>,
    pub block_number: u64,
    pub transaction_hash: H256, // Useful for debugging and idempotency
}

#[derive(Clone)]
pub struct EthereumClient<T: Transport> {
    web3: Web3<T>,
    delegate_registry_address: H160,
    event_signature: H256,
}

impl<T: Transport> EthereumClient<T> {
    pub fn new(web3: Web3<T>, delegate_registry_address: String) -> Result<Self, Error> {
        let address = H160::from_str(&delegate_registry_address)
            .map_err(|e| Error::Other(format!("Invalid address: {}", e)))?;

        // Calculate the event signature hash.  This is a constant, so you could
        // hardcode it, but calculating it is good practice.
        let event_signature_str = "DelegateSetChanged(address[])";
        let event_signature = web3::signing::keccak256(event_signature_str.as_bytes());

        Ok(Self {
            web3,
            delegate_registry_address: address,
            event_signature: H256::from(event_signature),
        })
    }

    pub async fn get_latest_block_number(&self) -> Result<u64, Error> {
        Ok(self.web3.eth().block_number().await?.as_u64())
    }

    // Get all logs, looping if necessary
    pub async fn get_all_logs(&self, from_block: U64, to_block: U64) -> Result<Vec<Log>, Error> {
        let filter = FilterBuilder::default()
            .address(vec![self.delegate_registry_address])
            .topics(Some(vec![self.event_signature]), None, None, None)
            .from_block(BlockNumber::Number(from_block))
            .to_block(BlockNumber::Number(to_block))
            .build();
        let result = self.web3.eth().logs(filter).await;
        let logs = match result {
            Ok(logs) => logs,
            Err(e) => {
                tracing::error!("Error getting logs {}", e);
                return Err(Error::EthereumRpcError(e));
            },
        };
        Ok(logs)
    }

    pub async fn get_delegate_set_changed_events(
        &self,
        from_block: u64,
        to_block: u64,
    ) -> Result<Vec<DelegateSetChangedEvent>, Error> {
        let logs = self
            .get_all_logs(U64::from(from_block), U64::from(to_block))
            .await?;
        let mut events = Vec::new();

        for log in logs {
            if let Some(block_number) = log.block_number {
                // Check if the event signature matches
                if let Some(topics) = log.topics.first() {
                    if *topics != self.event_signature {
                        continue; // Skip if not the correct event
                    }
                }

                // The event data should be a single, dynamic array of addresses.
                let data = log.data.0;

                // Ensure data length is a multiple of 32 (each address is 32 bytes with padding)
                if data.len() % 32 != 0 {
                    tracing::warn!("Invalid event data length: {}", data.len());
                    continue; // Skip malformed event data
                }

                let num_delegates = data.len() / 32;
                let mut delegates: Vec<H160> = Vec::with_capacity(num_delegates);

                for i in (0..data.len()).step_by(32) {
                    let address_bytes: [u8; 32] = data[i..i + 32]
                        .try_into()
                        .map_err(|_| Error::Other("slice with incorrect length".to_string()))?;

                    // Remove the 12 padding bytes
                    let address = H160::from_slice(&address_bytes[12..]);
                    delegates.push(address);
                }

                // Create the event
                let event = DelegateSetChangedEvent {
                    delegates,
                    block_number: block_number.as_u64(),
                    transaction_hash: log.transaction_hash.unwrap_or_default(), // Should always be present in valid logs
                };
                events.push(event);
            } else {
                tracing::warn!("Log without block number: {:?}", log);
            }
        }

        Ok(events)
    }
}
