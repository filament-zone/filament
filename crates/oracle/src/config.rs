use argh::FromArgs;

/// Filament oracle connecting hub and outpost
#[derive(FromArgs, Debug)]
pub struct Args {
    /// chain-id of the outpost
    #[argh(option, default = "String::from(\"test-1\")")]
    pub outpost_chain_id: String,

    /// outpost websocket endpoint
    #[argh(option, default = "String::from(\"ws://localhost:26657/websocket\")")]
    pub outpost_ws_endpoint: String,

    /// outpost grpc endpoint
    #[argh(option, default = "String::from(\"http://localhost:8090\")")]
    pub outpost_grpc_endpoint: String,

    /// hub rpc endpoint
    #[argh(option, default = "String::from(\"ws://localhost:12345\")")]
    pub hub_rpc_endpoint: String,

    /// location of the file on disk holding the outpost secret key
    #[argh(option, default = "String::from(\"./secret.key\")")]
    pub outpost_secret_key_file: String,

    /// location of the file on disk holding the hub secret key
    #[argh(
        option,
        default = "String::from(\"./test-data/keys/token_deployer_private_key.json\")"
    )]
    pub hub_secret_key_file: String,

    /// address of outpost contract
    #[argh(option)]
    pub outpost_addr: String,
}
