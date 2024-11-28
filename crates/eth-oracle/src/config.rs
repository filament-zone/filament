use argh::FromArgs;

/// Filament oracle connecting hub and ethereum
#[derive(FromArgs, Debug)]
pub struct Args {
    /// ethereum websocket endpoint
    #[argh(option, default = "String::from(\"ws://localhost:8545\")")]
    pub eth_ws_endpoint: String,

    /// ethereum grpc endpoint
    #[argh(option, default = "String::from(\"http://localhost:8545\")")]
    pub eth_rpc_endpoint: String,

    /// hub rpc endpoint
    #[argh(option, default = "String::from(\"http://localhost:12346\")")]
    pub hub_rpc_endpoint: String,

    /// location of the file on disk holding the eth secret key
    #[argh(option, default = "String::from(\"./secret.key\")")]
    pub eth_secret_key_file: String,

    /// location of the file on disk holding the hub secret key
    #[argh(
        option,
        default = "String::from(\"./test-data/keys/token_deployer_private_key.json\")"
    )]
    pub hub_secret_key_file: String,

    /// block height to start importing from, if 0 start from whatever the
    /// current block height is
    #[argh(option, default = "0u64")]
    pub eth_start_block: u64,

    /// block height to stop importing at, if 0 run until killed
    #[argh(option, default = "0u64")]
    pub eth_stop_block: u64,

    /// address of fila token contract
    #[argh(option)]
    pub eth_fila_token_addr: String,
}
