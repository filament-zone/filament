#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::ethereum::{DelegateSetChangedEvent, EthereumClient};
    use crate::tests::common::create_delegate_set_changed_event;
    use crate::tests::common::MockEthereumClient;
    use async_trait::async_trait;
    use std::str::FromStr;
    use web3::types::{H160, U64};

    // Helper function to create a mock EthereumClient
    fn setup_mock_client(
        events: Vec<DelegateSetChangedEvent>,
        next_block: u64,
    ) -> MockEthereumClient {
        MockEthereumClient {
            events,
            next_block,
            fail_get_latest_block: false, // Default value, can be overridden in tests
            fail_event_retrieval: false,
        }
    }

    #[tokio::test]
    async fn test_get_latest_block_number_success() {
        let client = setup_mock_client(vec![], 100);
        let block_number = client.get_latest_block_number().await.unwrap();
        assert_eq!(block_number, 100);
    }

    #[tokio::test]
    async fn test_get_latest_block_number_failure() {
        let mut client = setup_mock_client(vec![], 100);
        client.fail_get_latest_block = true;
        let result = client.get_latest_block_number().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_delegate_set_changed_events_success() {
        // Sample event data
        let event1 = create_delegate_set_changed_event(
            vec!["0x0000000000000000000000000000000000000001".to_string()],
            10,
        );

        let event2 = create_delegate_set_changed_event(
            vec!["0x0000000000000000000000000000000000000002".to_string()],
            12,
        );

        let client = setup_mock_client(vec![event1.clone(), event2.clone()], 15);
        // Fetch events within a specific block range
        let events = client
            .get_delegate_set_changed_events(10, 12)
            .await
            .unwrap();

        // Check that we got the correct events
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].block_number, event1.block_number);
        assert_eq!(events[0].delegates, event1.delegates);
        assert_eq!(events[1].block_number, event2.block_number);
    }

    #[tokio::test]
    async fn test_get_delegate_set_changed_events_empty() {
        let client = setup_mock_client(vec![], 15);
        let events = client.get_delegate_set_changed_events(10, 12).await;
        assert_eq!(events.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_get_delegate_set_changed_events_failure() -> Result<(), Error> {
        let mut client = setup_mock_client(vec![], 15);
        client.fail_event_retrieval = true;
        let result = client.get_delegate_set_changed_events(1, 1).await;
        assert!(result.is_err());
        Ok(())
    }
}
