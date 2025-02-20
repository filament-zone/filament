## Usage

The relayer provides a command-line interface with the following commands:

*   **`start`:** Starts the relayer.
*   **`query`:** Queries the current relayer status (last processed block and delegate powers).
*   **`reset`:** Resets the relayer's database (clears all stored data).

**Starting the Relayer:**

```bash
cargo run --bin filament-relayer -- start
```

You can optionally specify a starting block number:

```bash
cargo run --bin filament-relayer -- start --block-number 12345
```

If no block number is specified, the relayer will start from the last processed block stored in the database.  If the database is empty, it will start from the `genesis_block` specified in `config.toml`.

**Querying the Relayer Status:**

```bash
cargo run --bin filament-relayer -- query
```

This will output the last processed block and the current voting power of each delegate known to the relayer.

**Resetting the Relayer:**

```bash
cargo run --bin filament-relayer -- reset
```

This will *delete* all data in the relayer's database.  Use this with caution!  It's useful for restarting the relayer from scratch or for testing.

**Logging:**

The relayer uses the `tracing` crate for logging.  You can control the log level using the `RUST_LOG` environment variable.  For example, to see more detailed logs:

```bash
RUST_LOG=debug cargo run --bin filament-relayer -- start
```

Common log levels are: `error`, `warn`, `info`, `debug`, and `trace`.

**Environment Variables (Recommended for Production):**

Instead of hardcoding sensitive information in `config.toml`, use environment variables:

```bash
# Set environment variables (example - use your actual values)
export ETHEREUM_RPC_URL="http://your-ethereum-node:8545"
export HUB_URL="http://your-hub-url"
export DELEGATE_REGISTRY_ADDRESS="0xYourDelegateRegistryAddress"
export HUB_PRIVATE_KEY="0xYourHubPrivateKey" # AGAIN, HANDLE SECURELY!
export DATABASE_PATH="relayer.db" # or a different path

# Then, modify your config.toml to read from environment variables:
# (You can use a crate like `dotenv` to load from a .env file)

# ethereum_rpc_url = "${ETHEREUM_RPC_URL}"
# hub_url = "${HUB_URL}"
# delegate_registry_address = "${DELEGATE_REGISTRY_ADDRESS}"
# hub_private_key = "${HUB_PRIVATE_KEY}"
# database_path = "${DATABASE_PATH}"
# ... (keep the other settings as they are)
```

You'll need a way to parse environment variables inside the toml file. Alternatively, you can load environment variables directly in your Rust code using `std::env::var`.

## Testing

The crate includes comprehensive unit and integration tests.  You can run them with:

```bash
cargo test
```
