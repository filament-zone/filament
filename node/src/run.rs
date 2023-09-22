use std::net::SocketAddr;

use console_subscriber::ConsoleLayer;
use directories::BaseDirs;
use eyre::WrapErr as _;
use metrics_exporter_prometheus::PrometheusBuilder;
use metrics_tracing_context::{MetricsLayer, TracingContextLayer};
use metrics_util::layers::Stack;
use penumbra_storage::Storage;
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _, EnvFilter};

use crate::{Config, Consensus, Info, Mempool, Snapshot};

pub async fn run(cfg: Config) -> eyre::Result<()> {
    let dirs = BaseDirs::new().expect("failed to construct base directories");

    let console_layer = ConsoleLayer::builder().with_default_env().spawn();
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .expect("failed to set up env filter");
    let fmt_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_target(true)
        .with_thread_ids(true);
    let metrics_layer = MetricsLayer::new();

    tracing_subscriber::registry()
        .with(console_layer)
        .with(filter_layer)
        .with(fmt_layer)
        .with(metrics_layer)
        .init();

    let (recorder, exporter) = PrometheusBuilder::new()
        .with_http_listener(
            format!("{}:{}", cfg.host, cfg.metrics_port)
                .parse::<SocketAddr>()
                .expect("failed to parse metrics addr"),
        )
        .build()
        .expect("failed to build prometheus recorder");

    Stack::new(recorder)
        .push(TracingContextLayer::only_allow(["chain_id", "role"]))
        .install()
        .expect("failed to install recorder");

    tracing::info!("starting pulzaard");

    let storage = Storage::load(dirs.data_dir().join("pulzaar/devnet/pulzaard/rocksdb"))
        .await
        .map_err(|e| eyre::eyre!(e))
        .wrap_err("unable to initialise RocksDB storage")?;

    let consensus = Consensus::new(storage.clone()).await?;
    let info = Info::new(storage.clone()).await?;
    let mempool = Mempool::new(storage).await?;
    let snapshot = Snapshot::new().await?;

    let abci_fut = tower_abci::v034::Server::builder()
        .consensus(consensus)
        .mempool(mempool)
        .snapshot(snapshot)
        .info(info)
        .finish()
        .unwrap()
        .listen(format!("{}:{}", cfg.host, cfg.abci_port));
    let abci_server = tokio::task::Builder::new()
        .name("abci_server")
        .spawn(abci_fut)
        .expect("failed to spawn abci server");

    register_metrics();

    tokio::select! {
        res = exporter => res?,
        res = abci_server => res?.map_err(|e| eyre::eyre!(e))?,
    };

    Ok(())
}

fn register_metrics() {
    penumbra_storage::register_metrics();
}
