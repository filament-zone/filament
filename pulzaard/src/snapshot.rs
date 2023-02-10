use std::{pin::Pin, task::Poll};

use futures::{Future, FutureExt as _};
use tendermint::abci::{request, response, SnapshotRequest, SnapshotResponse};
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::PollSender;
use tower_abci::BoxError;
use tower_service::Service;
use tracing::{error_span, instrument, Instrument as _, Span};

use crate::RequestExt as _;

#[derive(Debug)]
struct Message {
    req: SnapshotRequest,
    res: oneshot::Sender<SnapshotResponse>,
    span: Span,
}

#[derive(Clone, Debug)]
pub struct Snapshot {
    queue: PollSender<Message>,
}

impl Snapshot {
    pub async fn new() -> eyre::Result<Self> {
        let (tx, rx) = mpsc::channel(128);

        tokio::task::Builder::new()
            .name("snapshot::Worker")
            .spawn(Worker::new(rx).run())
            .expect("failed to spawn snapshot worker");

        Ok(Self {
            queue: PollSender::new(tx),
        })
    }
}

impl Service<SnapshotRequest> for Snapshot {
    type Error = BoxError;
    type Future =
        Pin<Box<dyn Future<Output = Result<SnapshotResponse, BoxError>> + Send + 'static>>;
    type Response = SnapshotResponse;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: SnapshotRequest) -> Self::Future {
        if self.queue.is_closed() {
            return async move {
                Err(eyre::eyre!("snapshot worker terminated or panicked").into())
            }.boxed();
        }

        let span = req.create_span();
        let span = error_span!(parent: &span, "app", role = "snapshot");
        let (tx, rx) = oneshot::channel();

        self.queue
            .send_item(Message { req, res: tx, span })
            .expect("called without poll_ready");

        async move {
            rx.await
                .map_err(|_| eyre::eyre!("snapshot worker terminated or panicked").into())
        }
        .boxed()
    }
}

struct Worker {
    queue: mpsc::Receiver<Message>,
}

impl Worker {
    #[instrument(skip(queue), name = "snapshot::Worker::new")]
    fn new(queue: mpsc::Receiver<Message>) -> Self {
        Self { queue }
    }

    async fn run(mut self) -> eyre::Result<()> {
        while let Some(Message { req, res, span }) = self.queue.recv().await {
            let _ = res.send(match req {
                SnapshotRequest::ListSnapshots => SnapshotResponse::ListSnapshots(
                    self.list_snapshots()
                        .instrument(span)
                        .await
                        .expect("list_snapshots failed"),
                ),
                SnapshotRequest::OfferSnapshot(offer_snapshot) => SnapshotResponse::OfferSnapshot(
                    self.offer_snapshot(offer_snapshot)
                        .instrument(span)
                        .await
                        .expect("list_snapshots failed"),
                ),
                SnapshotRequest::LoadSnapshotChunk(load_snapshot_chunk) => {
                    SnapshotResponse::LoadSnapshotChunk(
                        self.load_snapshot_chunk(load_snapshot_chunk)
                            .instrument(span)
                            .await
                            .expect("list_snapshots failed"),
                    )
                },
                SnapshotRequest::ApplySnapshotChunk(apply_snaphshot_chunk) => {
                    SnapshotResponse::ApplySnapshotChunk(
                        self.apply_snapshot_chunk(apply_snaphshot_chunk)
                            .instrument(span)
                            .await
                            .expect("list_snapshots failed"),
                    )
                },
            });
        }

        Ok(())
    }

    async fn list_snapshots(&mut self) -> eyre::Result<response::ListSnapshots> {
        todo!()
    }

    async fn offer_snapshot(
        &mut self,
        _offer_snapshot: request::OfferSnapshot,
    ) -> eyre::Result<response::OfferSnapshot> {
        todo!()
    }

    async fn load_snapshot_chunk(
        &mut self,
        _load_snapshot_chunk: request::LoadSnapshotChunk,
    ) -> eyre::Result<response::LoadSnapshotChunk> {
        todo!()
    }

    async fn apply_snapshot_chunk(
        &mut self,
        _apply_snapshot_chunk: request::ApplySnapshotChunk,
    ) -> eyre::Result<response::ApplySnapshotChunk> {
        todo!()
    }
}
