use std::{future::Future, pin::Pin};

use bytes::Bytes;
use futures::FutureExt as _;
use penumbra_storage::{Snapshot, Storage};
use pulzaar_app::App;
use tendermint::abci::{
    request::{self, CheckTxKind},
    response,
    MempoolRequest,
    MempoolResponse,
};
use tokio::sync::{mpsc, oneshot, watch};
use tokio_util::sync::PollSender;
use tower_abci::BoxError;
use tracing::{error_span, instrument, Instrument, Span};

use crate::RequestExt as _;

#[derive(Debug)]
pub struct Message {
    pub tx_bytes: Bytes,
    pub res: oneshot::Sender<eyre::Result<()>>,
    pub span: Span,
}

#[derive(Clone)]
pub struct Mempool {
    queue: PollSender<Message>,
}

impl Mempool {
    pub async fn new(storage: Storage) -> eyre::Result<Self> {
        let (tx, rx) = mpsc::channel(128);

        tokio::task::Builder::new()
            .name("mempool::Worker")
            .spawn(Worker::new(storage, rx).await?.run())
            .expect("failed to spawn mempool worker");

        Ok(Self {
            queue: PollSender::new(tx),
        })
    }
}

impl tower_service::Service<MempoolRequest> for Mempool {
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<MempoolResponse, BoxError>> + Send + 'static>>;
    type Response = MempoolResponse;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.queue.poll_reserve(cx).map_err(Into::into)
    }

    fn call(&mut self, req: MempoolRequest) -> Self::Future {
        if self.queue.is_closed() {
            return async move { Err(eyre::eyre!("mempool worker terminated or panicked").into()) }
                .boxed();
        }

        let span = req.create_span();
        let span = error_span!(parent: &span, "app", role = "mempool");
        let (tx, rx) = oneshot::channel();

        let MempoolRequest::CheckTx(request::CheckTx { tx: tx_bytes, kind }) = req;

        self.queue
            .send_item(Message {
                tx_bytes,
                res: tx,
                span: span.clone(),
            })
            .expect("called without poll_ready");

        async move {
            let _kind_str = match kind {
                CheckTxKind::New => "new",
                CheckTxKind::Recheck => "recheck",
            };

            let res = rx.await?;

            match res {
                Ok(()) => {
                    tracing::debug!("tx accepted");
                    // TODO(xla): Add counter in metrics for mempool checktx total.

                    Ok(MempoolResponse::CheckTx(response::CheckTx::default()))
                },
                Err(e) => {
                    tracing::debug!(err = e.to_string(), "tx rejected");
                    // TODO(xla): Add counter in metrics for mempool checktx total.

                    Ok(MempoolResponse::CheckTx(response::CheckTx {
                        code: 1.into(),
                        log: format!("{e:#}"),
                        ..Default::default()
                    }))
                },
            }
        }
        .instrument(span)
        .boxed()
    }
}

struct Worker {
    app: App,
    queue: mpsc::Receiver<Message>,
    snapshot_rx: watch::Receiver<Snapshot>,
}

impl Worker {
    #[instrument(skip(storage, queue), name = "mempool::Worker::new")]
    async fn new(storage: Storage, queue: mpsc::Receiver<Message>) -> eyre::Result<Self> {
        let app = App::new(storage.latest_snapshot());
        let snapshot_rx = storage.subscribe();

        Ok(Self {
            app,
            queue,
            snapshot_rx,
        })
    }

    pub async fn run(mut self) -> eyre::Result<()> {
        loop {
            tokio::select! {
                biased;

                change = self.snapshot_rx.changed() => {
                    if let Ok(()) = change {
                        let state = self.snapshot_rx.borrow().clone();
                        tracing::debug!(height = ?state.version(), "resetting ephemeral mempool state");
                        self.app = App::new(state);
                    } else {
                        tracing::debug!("state notification channel closed, shutting down");
                        todo!()
                    }
                }
                message = self.queue.recv() => {
                    if let Some(Message { tx_bytes, res, span }) = message {
                        let _ = res.send(self.app.deliver_tx_bytes(tx_bytes.as_ref()).instrument(span).await.map(|_| ()));
                    } else {
                        tracing::debug!("message queue closed, shutting down");
                        return Ok(());
                    }
                }
            }
        }
    }
}
