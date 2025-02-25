mod tests {
    use crate::cli::CliCommand;
    use crate::common::{
        create_delegate_set_changed_event, MockDatabase, MockEthereumClient, MockHubClient,
    }; // Import from common
    use crate::config::Config;
    use crate::error::Error;
    use crate::relayer::Relayer;
    use std::str::FromStr;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;
    use web3::types::H160;

    // Helper function to create a test configuration
    fn create_test_config() -> Config {
        Config {
            ethereum_rpc_url: "http://localhost:8545".to_string(),
            hub_url: "http://localhost:3000".to_string(),
            delegate_registry_address: "0x1234567890123456789012345678901234567890".to_string(),
            polling_interval_seconds: 1,
            database_path: "test.db".to_string(),
            hub_private_key: "0xabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
                .to_string(),
            max_retries: 3,
            retry_backoff_seconds: 1,
            genesis_block: 1000,
            batch_size: 100,
        }
    }

    // What is this test about?
    // it starts the
    #[test]
    fn test_relayer_process_events() -> Result<(), Error> {
        let config = create_test_config();
        let mut mock_hub_client = MockHubClient::default(); // Create the MOCK
        let delegate = H160::from_str("0x0000000000000000000000000000000000000001").unwrap();

        // *** SET UP MOCK BEHAVIOR ***
        // We *do* access sent_transactions here, because this is the CONCRETE mock,
        // *before* we create the Relayer.
        mock_hub_client.sent_transactions.lock().unwrap().push((
            "0x0000000000000000000000000000000000000001".to_string(),
            100_u64,
        ));
        mock_hub_client.set_confirmed("0x0000000000000000000000000000000000000001");

        let mock_ethereum_client = MockEthereumClient {
            events: vec![create_delegate_set_changed_event(
                vec!["0x0000000000000000000000000000000000000001".to_string()],
                1001,
            )],
            next_block: 1002,
            fail_get_latest_block: false,
            fail_event_retrieval: false,
        };

        let database = MockDatabase::default();

        // NOW create the Relayer, passing in the BOXED mock.
        let relayer = Relayer::new(
            config,
            Box::new(mock_ethereum_client),
            Box::new(mock_hub_client), // Box it HERE
            Arc::new(database),
        );

        // Start relayer
        relayer.start(Some(1001))?;

        // Give the relayer some time to process
        std::thread::sleep(std::time::Duration::from_secs(3));

        // Check the database (using trait methods - good!)
        assert_eq!(relayer.database.get_last_processed_block()?, Some(1001));

        // XXX: This doesn't make sense
        // let sent_txs = relayer.hub_client.get_sent_transactions().await?; // <--- USE THE TRAIT METHOD
        //assert_eq!(sent_txs.len(), 1);

        Ok(())
    }

    // Other tests (corrected to use setup_mock_relayer)
    #[test]
    fn test_relayer_start_from_genesis() -> Result<(), Error> {
        let relayer = setup_mock_relayer();
        relayer.start(Some(relayer.config.genesis_block))?;
        assert_eq!(
            relayer.database.get_last_processed_block()?,
            Some(relayer.config.genesis_block)
        );
        Ok(())
    }

    #[test]
    fn test_relayer_restart() -> Result<(), Error> {
        let event = create_delegate_set_changed_event(
            vec!["0x0000000000000000000000000000000000000001".to_string()],
            1001,
        );

        // Modify MockEthereumClient directly *before* creating Relayer
        let mut mock_ethereum_client = MockEthereumClient::default();
        mock_ethereum_client.events = vec![event.clone()];
        mock_ethereum_client.next_block = 1002;

        let mut mock_hub_client = MockHubClient::default();
        mock_hub_client.sent_transactions.lock().unwrap().push((
            "0x0000000000000000000000000000000000000001".to_string(),
            100_u64,
        ));
        mock_hub_client.set_confirmed("0x0000000000000000000000000000000000000001");

        // Now create Relayer with the modified mock
        let relayer = Relayer::new(
            create_test_config(), // Create config directly
            Box::new(mock_ethereum_client),
            Box::new(mock_hub_client),
            Arc::new(MockDatabase::default()),
        );

        relayer.start(Some(1001))?;
        thread::sleep(Duration::from_secs(2));
        assert_eq!(relayer.database.get_last_processed_block()?, Some(1001));

        Ok(())
    }

    #[test]
    fn test_relayer_start_command() -> Result<(), Error> {
        let relayer = setup_mock_relayer();
        relayer.run_command(CliCommand::Start {
            block_number: (Some(relayer.config.genesis_block)),
        })?;
        assert_eq!(
            relayer.database.get_last_processed_block()?,
            Some(relayer.config.genesis_block)
        );
        Ok(())
    }

    #[test]
    fn test_relayer_query_command() -> Result<(), Error> {
        let relayer = setup_mock_relayer();
        let delegate = H160::from_str("0x0000000000000000000000000000000000000001").unwrap();
        let power = 100_u64;
        relayer.database.save_last_processed_block(12345)?;
        relayer.database.update_delegate_power(&delegate, power)?;
        relayer.run_command(CliCommand::Query)?;
        Ok(())
    }

    #[test]
    fn test_relayer_reset_command() -> Result<(), Error> {
        let relayer = setup_mock_relayer();
        let delegate = H160::from_str("0x0000000000000000000000000000000000000001").unwrap();
        relayer.database.save_last_processed_block(12345)?;
        relayer.database.update_delegate_power(&delegate, 100)?;
        relayer.run_command(CliCommand::Reset)?;
        assert_eq!(relayer.database.get_last_processed_block()?, None);
        assert!(relayer.database.get_all_delegate_powers()?.is_empty());
        Ok(())
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
            Arc::new(database),
        )
    }
}
