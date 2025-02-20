#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::hub::HubClient;
    use crate::tests::common::MockHubClient;
    use std::str::FromStr;

    // Helper function to set up a mock HubClient
    fn setup_mock_client() -> MockHubClient {
        MockHubClient::default()
    }

    #[tokio::test]
    async fn test_update_voting_power_success() {
        let mut client = setup_mock_client();

        let address = "0x0000000000000000000000000000000000000001".to_string();
        let power = 100;

        let result = client.update_voting_power(address.clone(), power).await;
        assert!(result.is_ok());

        // Verify that the transaction was sent (using the mock's internal state)
        let sent_transactions = client.sent_transactions.lock().unwrap();
        assert_eq!(sent_transactions.len(), 1);
        assert_eq!(sent_transactions[0], (address, power));
    }

    #[tokio::test]
    async fn test_update_voting_power_failure() {
        let mut client = setup_mock_client();
        client.fail_update_voting_power = true; // Set the mock to fail

        let address = "0x0000000000000000000000000000000000000001".to_string();
        let power = 100;
        let result = client.update_voting_power(address, power).await;
        assert!(result.is_err()); // Expect an error
    }

    #[tokio::test]
    async fn test_get_tx_status_confirmed() -> Result<(), Error> {
        let mut client = setup_mock_client();
        let tx_hash = "0x1234".to_string();
        client.set_confirmed(&tx_hash);

        let status = client.get_tx_status(&tx_hash).await?;
        assert!(status.is_some());
        Ok(())
    }

    #[tokio::test]
    async fn test_get_tx_status_not_found() -> Result<(), Error> {
        let client = setup_mock_client();

        let tx_hash = "0x1234".to_string();
        let status = client.get_tx_status(&tx_hash).await?;
        assert!(status.is_none());
        Ok(())
    }
}
