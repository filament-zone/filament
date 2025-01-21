use std::{fs, sync::Arc};

use alloy::{
    network::EthereumWallet,
    providers::{Provider, ProviderBuilder, WsConnect},
    signers::local::PrivateKeySigner,
    transports::http::reqwest::Url,
};
use eyre::{eyre, Result};

mod config;
mod delegate_registry;
mod eth;
mod filament_token;
mod hub;

use filament_token::FilamentTokenSync;
use futures::FutureExt;
use sov_cli::wallet_state::PrivateKeyAndAddress;
use sov_modules_api::{CryptoSpec, PrivateKey, PublicKey, Spec};
use tokio::sync::{watch, Mutex};
use tracing::trace;

// XXX: Right now things are setup in such a way that blocks/slots are streamed to
//      consumers. It might be better to invert that relationship and have consumers
//      request blocks/slots instead. That should make it easier to switch between
//      follow latest and catch up mode and also allow the consumer to move at its
//      own pace especially when waiting for transactions etc.

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = argh::from_env::<config::Args>();

    let rpc_url: Url = args.eth_rpc_endpoint.parse()?;

    let eth_signing_key = {
        let mut raw = fs::read(args.eth_secret_key_file.clone())?;
        raw.truncate(raw.len() - 1); // why???
        hex::decode(raw)?
    };
    let eth_signer: PrivateKeySigner = PrivateKeySigner::from_slice(&eth_signing_key)?;
    trace!("eth_signer: {:?}", eth_signer);
    let eth_wallet = EthereumWallet::from(eth_signer);

    let eth_block_provider = ProviderBuilder::new().on_http(rpc_url.clone());

    let eth_ws = WsConnect::new(args.eth_ws_endpoint);

    // hub

    // XXX: maybe get rid of sov_cli
    let key_and_address: PrivateKeyAndAddress<hub::FilaSpec> = serde_json::from_str(
        &std::fs::read_to_string(args.hub_secret_key_file.clone())
            .expect("Unable to read file to string"),
    )?;
    trace!("hub address: {:?}", key_and_address.address);
    let credential_id = key_and_address
        .private_key
        .pub_key()
        .credential_id::<<<hub::FilaSpec as Spec>::CryptoSpec as CryptoSpec>::Hasher>(
    );

    let hub_account = Arc::new(Mutex::new(hub::Account {
        credential_id,
        nonce: 0,
    }));

    let h = hub::Hub::new(args.hub_rpc_endpoint.clone(), hub_account.clone(), key_and_address.private_key)?;
    let slot = h.last_slot().await?;
    trace!(slot = ?slot);

    let fts = Arc::new(FilamentTokenSync::new(
        args.eth_fila_token_addr.parse()?,
        rpc_url.clone(),
        eth_wallet.clone(),
    ));

    trace!("getting latest block for startup");
    let (last_block_sender, last_block_recv) = match eth_block_provider
        .get_block_by_number(alloy::eips::BlockNumberOrTag::Latest, true)
        .await
    {
        Ok(Some(block)) => {
            // println!("block: {:?}", res.block_id);
            Ok(watch::channel(block))
        },
        Ok(None) => Err(eyre!("did not get a block")),
        Err(e) => Err(eyre!("failed to poll latest block: {}", e)),
    }?;

    let mut token_sync_handle =
        tokio::spawn(filament_token::listener(fts, last_block_recv.clone())).fuse();

    trace!("spawning block_watcher");
    let mut latest_block_handle = tokio::spawn(eth::block_watcher(
        eth_ws.clone(),
        last_block_sender.clone(),
        args.eth_start_block,
        args.eth_stop_block,
    ))
    .fuse();

    let (last_slot_sender, last_slot_recv) = watch::channel(0);

    let mut hub_account_handle = tokio::spawn(hub::account_watcher(
        args.hub_rpc_endpoint.clone(),
        credential_id,
        hub_account,
        last_slot_recv.clone(),
    ))
    .fuse();

    let mut latest_slot_handle = tokio::spawn(hub::slot_watcher(
        args.hub_rpc_endpoint.clone(),
        last_slot_sender.clone(),
    ))
    .fuse();

    loop {
        tokio::select! {
            join = &mut latest_block_handle => join??,
            join = &mut latest_slot_handle => join??,
            join = &mut token_sync_handle => join??,
            join = &mut hub_account_handle => join??,
            else => break
        }
    }

    Ok(())
}
