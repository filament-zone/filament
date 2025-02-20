#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::database::MockDatabase;
    use crate::error::Error;
    use crate::ethereum::MockEthereumClient;
    use crate::hub::MockHubClient;
    use crate::relayer::Relayer;
    use crate::tests::common::create_delegate_set_changed_event;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::sync::Mutex;
    use std::thread;
    use std::time::Duration;

    // Helper function to create a test configuration
    fn create_test_config() -> Config {
        Config {
            ethereum_rpc_url: "http://localhost:8545".to_string(),
            hub_url: "http://localhost:3000".to_string(),
            delegate_registry_address: "0x1234567890123456789012345678901234567890".to_string(), // Replace with a valid address
            polling_interval_seconds: 1,
            database_path: "test.db".to_string(),
            hub_private_key: "0xabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
                .to_string(), // Replace
            max_retries: 3,
            retry_backoff_seconds: 1,
            genesis_block: 1000,
            batch_size: 100,
        }
    }
    // Helper function to set up a mock environment for testing the relayer
    fn setup_mock_relayer() -> Relayer {
        let config = create_test_config();
        let ethereum_client = MockEthereumClient::default();
        let hub_client = MockHubClient::default();
        let database = MockDatabase::default();

        Relayer::new(
            config,
            Box::new(ethereum_client),
            Box::new(hub_client),
            Box::new(database),
        )
    }

    #[test]
    fn test_relayer_start_from_genesis() -> Result<(), Error> {
        let mut relayer = setup_mock_relayer();

        // Set up mock behavior: no events yet

        // Start the relayer from genesis
        relayer.start(Some(relayer.config.genesis_block))?;

        // Check that the last processed block is the genesis block
        assert_eq!(
            relayer.database.get_last_processed_block()?,
            Some(relayer.config.genesis_block)
        );

        Ok(())
    }

    #[test]
    fn test_relayer_process_events() -> Result<(), Error> {
        let mut relayer = setup_mock_relayer();

        let event = create_delegate_set_changed_event(
            vec!["0x0000000000000000000000000000000000000001".to_string()],
            1001,
        );
        // Set up mock behavior: return a single event

        relayer.ethereum_client.events = vec![event.clone()];
        relayer.ethereum_client.next_block = 1002;

        // Start relayer from last_processed_block + 1
        relayer.start(Some(1001))?;

        // Give the relayer some time to process the event
        std::thread::sleep(std::time::Duration::from_secs(2));
        // Check the database for the updates

        assert_eq!(relayer.database.get_last_processed_block()?, Some(1001));
        //assert_eq!(
        //relayer.database.get_delegate_power(&H160::from_str("0x0000000000000000000000000000000000000001").unwrap())?,
        //Some(100)
        //);

        Ok(())
    }

    #[test]
    fn test_relayer_restart() -> Result<(), Error> {
        let mut relayer = setup_mock_relayer();
        let event = create_delegate_set_changed_event(
            vec!["0x0000000000000000000000000000000000000001".to_string()],
            1001,
        );
        // Set up mock behavior: return a single event
        relayer.ethereum_client.events = vec![event.clone()];
        relayer.ethereum_client.next_block = 1002;

        // Start relayer from last_processed_block + 1
        relayer.start(Some(1001))?;

        // Give the relayer some time to process the event
        thread::sleep(Duration::from_secs(2));
        // Check the database for the updates

        assert_eq!(relayer.database.get_last_processed_block()?, Some(1001));
        //assert_eq!(
        //relayer.database.get_delegate_power(&H160::from_str("0x0000000000000000000000000000000000000001").unwrap())?,
        //Some(100)
        //);

        Ok(())
    }

    #[test]
    fn test_relayer_start_command() -> Result<(), Error> {
        let mut relayer = setup_mock_relayer();

        // Set up mock behavior: no events yet

        // Start the relayer from genesis
        relayer.run_command(CliCommand::Start {
            block_number: (Some(relayer.config.genesis_block)),
        })?;

        // Check that the last processed block is the genesis block
        assert_eq!(
            relayer.database.get_last_processed_block()?,
            Some(relayer.config.genesis_block)
        );

        Ok(())
    }

    #[test]
    fn test_relayer_query_command() -> Result<(), Error> {
        // Create a temporary directory for the test database
        let temp_dir = tempfile::tempdir()?;
        let db_path = temp_dir.path().join("test.db");
        let mut relayer = setup_mock_relayer();
        let delegate = H160::from_str("0x0000000000000000000000000000000000000001").unwrap();
        let power = 100_u64;

        // Insert some sample data
        relayer.database.save_last_processed_block(12345)?;
        relayer.database.update_delegate_power(&delegate, power)?;

        // Call the query command
        relayer.run_command(CliCommand::Query)?;
        Ok(())
    }

    #[test]
    fn test_relayer_reset_command() -> Result<(), Error> {
        let mut relayer = setup_mock_relayer();
        let delegate = H160::from_str("0x0000000000000000000000000000000000000001").unwrap();
        // Insert some sample data
        relayer.database.save_last_processed_block(12345)?;
        relayer.database.update_delegate_power(&delegate, 100)?;

        // Call the reset command
        relayer.run_command(CliCommand::Reset)?;

        // Check that the database is empty
        assert_eq!(relayer.database.get_last_processed_block()?, None);
        assert!(relayer.database.get_all_delegate_powers()?.is_empty());
        Ok(())
    }
}
