use std::{future::Future, pin::Pin};

use futures::FutureExt as _;
use penumbra_storage::Storage;
use pulzaar_app::App;
use pulzaar_chain::{genesis::AppState, ChainId, ChainParameters};
use tendermint::abci::{self, request, response, ConsensusRequest, ConsensusResponse};
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::PollSender;
use tower_abci::BoxError;
use tower_service::Service;
use tracing::{error_span, instrument, Instrument as _, Span};

use crate::RequestExt as _;

#[derive(Debug)]
struct Message {
    req: ConsensusRequest,
    res: oneshot::Sender<ConsensusResponse>,
    span: Span,
}

#[derive(Clone)]
pub struct Consensus {
    queue: PollSender<Message>,
}

impl Consensus {
    pub async fn new(storage: Storage) -> eyre::Result<Self> {
        let (tx, rx) = mpsc::channel(128);

        tokio::task::Builder::new()
            .name("consensus::Worker")
            .spawn(Worker::new(storage, rx).run())
            .expect("failed to spawn consensus worker");

        Ok(Self {
            queue: PollSender::new(tx),
        })
    }
}

impl Service<ConsensusRequest> for Consensus {
    type Error = BoxError;
    type Future =
        Pin<Box<dyn Future<Output = Result<ConsensusResponse, BoxError>> + Send + 'static>>;
    type Response = ConsensusResponse;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.queue.poll_reserve(cx).map_err(Into::into)
    }

    fn call(&mut self, req: ConsensusRequest) -> Self::Future {
        if self.queue.is_closed() {
            return async move {
                Err(eyre::eyre!("consensus worker terminated or panicked").into())
            }.boxed();
        }

        let span = req.create_span();
        let span = error_span!(parent: &span, "app", role = "consensus");
        let (tx, rx) = oneshot::channel();

        self.queue
            .send_item(Message { req, res: tx, span })
            .expect("called without poll_ready");

        async move {
            rx.await
                .map_err(|_| eyre::eyre!("consensus worker terminated or panicked").into())
        }
        .boxed()
    }
}

struct Worker {
    app: App,
    queue: mpsc::Receiver<Message>,
    storage: Storage,
}

impl Worker {
    #[instrument(skip(storage, queue), name = "consensus::Worker::new")]
    fn new(storage: Storage, queue: mpsc::Receiver<Message>) -> Self {
        let app = App::new(storage.latest_snapshot());

        Self {
            app,
            queue,
            storage,
        }
    }

    async fn run(mut self) -> eyre::Result<()> {
        while let Some(Message { req, res, span }) = self.queue.recv().await {
            let _ = res.send(match req {
                ConsensusRequest::InitChain(init_chain) => ConsensusResponse::InitChain(
                    self.init_chain(init_chain)
                        .instrument(span)
                        .await
                        .expect("init_chain failed"),
                ),
                ConsensusRequest::BeginBlock(begin_block) => ConsensusResponse::BeginBlock(
                    self.begin_block(begin_block)
                        .instrument(span)
                        .await
                        .expect("begin_block failed"),
                ),
                ConsensusRequest::DeliverTx(deliver_tx) => ConsensusResponse::DeliverTx(
                    self.deliver_tx(deliver_tx)
                        .instrument(span)
                        .await
                        .expect("deliver_tx failed"),
                ),
                ConsensusRequest::EndBlock(end_block) => ConsensusResponse::EndBlock(
                    self.end_block(end_block)
                        .instrument(span)
                        .await
                        .expect("end_block failed"),
                ),
                ConsensusRequest::Commit => ConsensusResponse::Commit(
                    self.commit().instrument(span).await.expect("commit failed"),
                ),
            });
        }

        Ok(())
    }

    async fn init_chain(
        &mut self,
        init_chain: request::InitChain,
    ) -> eyre::Result<response::InitChain> {
        // TODO(xla): Deserialize app state.
        let app_state = AppState {
            allocations: vec![],
            chain_parameters: ChainParameters {
                chain_id: ChainId::try_from("pulzaar-devnet".to_string())?,
                epoch_duration: 0,
            },
        };

        // TODO(xla): Error if storage is not at height 0.

        self.app.init_chain(&app_state).await;

        // TODO(xla): Extract validators from app state.
        let validators = vec![];

        let app_hash = self.app.commit(self.storage.clone()).await;

        tracing::info!(
            consensus_params = ?init_chain.consensus_params,
            ?validators,
            app_hash = ?app_hash,
            "chain initialized",
        );

        Ok(response::InitChain {
            consensus_params: Some(init_chain.consensus_params),
            validators,
            app_hash: app_hash.0.to_vec().try_into()?,
        })
    }

    async fn begin_block(
        &mut self,
        begin_block: request::BeginBlock,
    ) -> eyre::Result<response::BeginBlock> {
        tracing::info!(time = ?begin_block.header.time, "beginning block");
        tracing::trace!(begin_block = ?begin_block);

        let events = self.app.begin_block(&begin_block).await;
        trace_events(&events);

        Ok(response::BeginBlock { events })
    }

    async fn deliver_tx(
        &mut self,
        deliver_tx: request::DeliverTx,
    ) -> eyre::Result<response::DeliverTx> {
        match self.app.deliver_tx_bytes(deliver_tx.tx.as_ref()).await {
            Ok(events) => {
                trace_events(&events);

                Ok(response::DeliverTx {
                    events,
                    ..Default::default()
                })
            },
            Err(e) => {
                tracing::debug!(?e, "deliver tx failed");

                Ok(response::DeliverTx {
                    code: 1,
                    log: format!("{e:}"),
                    ..Default::default()
                })
            },
        }
    }

    async fn end_block(
        &mut self,
        end_block: request::EndBlock,
    ) -> eyre::Result<response::EndBlock> {
        tracing::info!(height = end_block.height, "ending block");

        let (validator_updates, consensus_param_updates, events) =
            self.app.end_block(&end_block).await;
        trace_events(&events);

        Ok(response::EndBlock {
            validator_updates,
            consensus_param_updates,
            events,
        })
    }

    async fn commit(&mut self) -> eyre::Result<response::Commit> {
        let app_hash = self.app.commit(self.storage.clone()).await;
        tracing::info!(?app_hash, "committed block");

        Ok(response::Commit {
            data: app_hash.0.to_vec().into(),
            retain_height: 0u32.into(),
        })
    }
}

fn trace_events(events: &[abci::Event]) {
    for event in events {
        let span = tracing::trace_span!("event", kind = event.kind);
        span.in_scope(|| {
            for attr in &event.attributes {
                tracing::trace!(key = attr.key, value = attr.value, index = attr.index);
            }
        })
    }
}
