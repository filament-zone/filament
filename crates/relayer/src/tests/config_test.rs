#[cfg(test)]
mod tests {
    use crate::config::Config;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn test_load_config_success() {
        // Create a temporary directory
        let tmp_dir = TempDir::new().unwrap();
        let config_path = tmp_dir.path().join("config.toml");

        // Write a sample config file
        let mut file = File::create(&config_path).unwrap();
        writeln!(
            file,
            r#"
            ethereum_rpc_url = "http://localhost:8545"
            hub_url = "http://localhost:3000"
            delegate_registry_address = "0x1234567890123456789012345678901234567890"
            polling_interval_seconds = 5
            database_path = "test.db"
            hub_private_key = "0xabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
            max_retries = 3
            retry_backoff_seconds = 2
            genesis_block = 1000
            batch_size = 100
        "#
        )
        .unwrap();

        // Load the configuration
        let config = Config::load(&config_path).unwrap();

        // Assert the values
        assert_eq!(config.ethereum_rpc_url, "http://localhost:8545");
        assert_eq!(config.hub_url, "http://localhost:3000");
        assert_eq!(
            config.delegate_registry_address,
            "0x1234567890123456789012345678901234567890"
        );
        assert_eq!(config.polling_interval_seconds, 5);
        assert_eq!(config.database_path, "test.db");
        assert_eq!(
            config.hub_private_key,
            "0xabcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
        );
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.retry_backoff_seconds, 2);
        assert_eq!(config.genesis_block, 1000);
        assert_eq!(config.batch_size, 100)
    }

    #[test]
    fn test_load_config_missing_file() {
        let config_path = Path::new("nonexistent_config.toml");
        let result = Config::load(config_path);
        assert!(result.is_err()); // Expect an error
    }

    #[test]
    fn test_load_config_invalid_toml() {
        let tmp_dir = TempDir::new().unwrap();
        let config_path = tmp_dir.path().join("invalid_config.toml");
        let mut file = File::create(&config_path).unwrap();
        writeln!(file, "invalid toml content").unwrap(); // Invalid TOML

        let result = Config::load(&config_path);
        assert!(result.is_err());
    }
}
