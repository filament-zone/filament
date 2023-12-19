use std::pin::Pin;

use cnidarium::Storage;
use filament_app::{App, AppHashRead as _};
use futures::{Future, FutureExt as _};
use tendermint::{
    block::Height,
    v0_34::abci::{request, response, InfoRequest, InfoResponse},
};
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::PollSender;
use tower_abci::BoxError;
use tower_service::Service;
use tracing::{error_span, instrument, Instrument as _, Span};

use crate::RequestExt as _;

const ABCI_INFO_VERSION: &str = env!("CARGO_PKG_VERSION");
const APP_VERSION: u64 = 1;

#[derive(Debug)]
pub struct Message {
    pub req: InfoRequest,
    pub res: oneshot::Sender<InfoResponse>,
    pub span: Span,
}

#[derive(Clone)]
pub struct Info {
    queue: PollSender<Message>,
}

impl Info {
    pub async fn new(storage: Storage) -> eyre::Result<Self> {
        let (tx, rx) = mpsc::channel(128);

        tokio::task::Builder::new()
            .name("info::Worker")
            .spawn(Worker::new(storage, rx).run())
            .expect("failed to spawn info worker");

        Ok(Self {
            queue: PollSender::new(tx),
        })
    }
}

impl Service<InfoRequest> for Info {
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<InfoResponse, BoxError>> + Send + 'static>>;
    type Response = InfoResponse;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.queue.poll_reserve(cx).map_err(Into::into)
    }

    fn call(&mut self, req: InfoRequest) -> Self::Future {
        if self.queue.is_closed() {
            return async move { Err(eyre::eyre!("info worker terminated or panicked").into()) }
                .boxed();
        }

        let span = req.create_span();
        let span = error_span!(parent: &span, "app", role = "info");
        let (tx, rx) = oneshot::channel();

        self.queue
            .send_item(Message { req, res: tx, span })
            .expect("called without poll_ready");

        async move {
            rx.await
                .map_err(|_| eyre::eyre!("info worker terminated or panicked").into())
        }
        .boxed()
    }
}

struct Worker {
    queue: mpsc::Receiver<Message>,
    storage: Storage,
}

impl Worker {
    #[instrument(skip(storage, queue), name = "info::Worker::new")]
    fn new(storage: Storage, queue: mpsc::Receiver<Message>) -> Self {
        Self { queue, storage }
    }

    async fn run(mut self) -> eyre::Result<()> {
        while let Some(Message { req, res, span }) = self.queue.recv().await {
            let _ = res.send(match req {
                InfoRequest::Info(info) => {
                    InfoResponse::Info(self.info(info).instrument(span).await.expect("info failed"))
                },
                InfoRequest::Query(query) => InfoResponse::Query(
                    self.query(query)
                        .instrument(span)
                        .await
                        .expect("query failed"),
                ),
                InfoRequest::Echo(echo) => {
                    InfoResponse::Echo(self.echo(echo).instrument(span).await.expect("echo failed"))
                },
                InfoRequest::SetOption(set_option) => InfoResponse::SetOption(
                    self.set_option(set_option)
                        .instrument(span)
                        .await
                        .expect("set_option failed"),
                ),
            });
        }

        Ok(())
    }

    async fn info(&mut self, info: request::Info) -> eyre::Result<response::Info> {
        let state = self.storage.latest_snapshot();
        tracing::info!(?info, version = ?state.version());

        let last_block_height = match state.version() {
            u64::MAX => 0,
            v => v,
        }
        .try_into()
        .unwrap();
        let app_hash = state.app_hash().await?.0.to_vec();

        Ok(response::Info {
            data: "filament".to_string(),
            version: ABCI_INFO_VERSION.to_string(),
            app_version: APP_VERSION,
            last_block_height,
            last_block_app_hash: app_hash.try_into()?,
        })
    }

    async fn query(&mut self, query: request::Query) -> eyre::Result<response::Query> {
        let state = if query.height == Height::from(0u32) {
            self.storage.latest_snapshot()
        } else {
            self.storage
                .snapshot(query.height.into())
                .ok_or(eyre::eyre!(
                    "snapshot for height {} not found",
                    query.height
                ))?
        };

        tracing::info!(?query, version = ?state.version());

        let query = request::Query {
            height: state.version().try_into()?,
            ..query
        };

        // TODO(tsenart): Support query.prove.
        App::new(state).query(&query).await
    }

    async fn echo(&mut self, _echo: request::Echo) -> eyre::Result<response::Echo> {
        todo!()
    }

    async fn set_option(
        &mut self,
        _set_option: request::SetOption,
    ) -> eyre::Result<response::SetOption> {
        todo!()
    }
}
