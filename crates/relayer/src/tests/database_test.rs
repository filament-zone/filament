#[cfg(test)]
mod tests {
    use crate::database::Database;
    use crate::error::Error;
    use std::collections::HashMap;
    use std::str::FromStr;
    use web3::types::H160;

    // Helper function to create a temporary database for testing
    fn setup_test_db() -> Result<Database, Error> {
        let temp_dir = tempfile::tempdir()?;
        let db_path = temp_dir.path().join("test.db");
        let db = Database::new(db_path.to_str().unwrap())?; // Convert PathBuf to &str
        Ok(db)
    }

    #[test]
    fn test_initialize_database() -> Result<(), Error> {
        let db = setup_test_db()?;

        // Check that the database is not initialized
        assert!(db.get_last_processed_block()?.is_none());

        // Initialize with a genesis block
        let genesis_block = 1000;
        db.initialize(genesis_block)?;

        // Verify that the genesis block is set
        assert_eq!(db.get_last_processed_block()?, Some(genesis_block));

        // Try initializing again (should not change the value)
        db.initialize(2000)?;
        assert_eq!(db.get_last_processed_block()?, Some(genesis_block));

        Ok(())
    }

    #[test]
    fn test_save_and_get_last_processed_block() -> Result<(), Error> {
        let db = setup_test_db()?;

        // Save a block number
        let block_number = 12345;
        db.save_last_processed_block(block_number)?;

        // Retrieve the block number
        let retrieved_block = db.get_last_processed_block()?;
        assert_eq!(retrieved_block, Some(block_number));

        Ok(())
    }

    #[test]
    fn test_update_and_get_delegate_power() -> Result<(), Error> {
        let db = setup_test_db()?;

        // Sample delegate address and power
        let delegate_address =
            H160::from_str("0x0000000000000000000000000000000000000001").unwrap();
        let power = 100;

        // Update the delegate's power
        db.update_delegate_power(&delegate_address, power)?;

        // Retrieve the power
        let retrieved_power = db.get_delegate_power(&delegate_address)?;
        assert_eq!(retrieved_power, Some(power));

        // Update again with a different power
        let new_power = 200;
        db.update_delegate_power(&delegate_address, new_power)?;
        let updated_power = db.get_delegate_power(&delegate_address)?;
        assert_eq!(updated_power, Some(new_power));

        Ok(())
    }
    #[test]
    fn test_get_all_delegate_powers() -> Result<(), Error> {
        let db = setup_test_db()?;

        // Insert some sample data
        let delegate1 = H160::from_str("0x0000000000000000000000000000000000000001").unwrap();
        let power1 = 100;
        let delegate2 = H160::from_str("0x0000000000000000000000000000000000000002").unwrap();
        let power2 = 200;

        db.update_delegate_power(&delegate1, power1)?;
        db.update_delegate_power(&delegate2, power2)?;

        // Retrieve all delegate powers
        let all_powers = db.get_all_delegate_powers()?;

        // Check that the retrieved data is correct
        let mut expected_powers = HashMap::new();
        expected_powers.insert(delegate1, power1);
        expected_powers.insert(delegate2, power2);

        assert_eq!(all_powers.len(), 2);
        assert_eq!(all_powers, expected_powers);

        Ok(())
    }

    #[test]
    fn test_clear_database() -> Result<(), Error> {
        let db = setup_test_db()?;
        let delegate_address =
            H160::from_str("0x0000000000000000000000000000000000000001").unwrap();
        db.update_delegate_power(&delegate_address, 1)?;
        db.save_last_processed_block(1)?;

        db.clear_database()?;
        assert!(db.get_all_delegate_powers()?.is_empty());
        Ok(())
    }
}
