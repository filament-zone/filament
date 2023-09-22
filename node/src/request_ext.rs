use sha2::{Digest as _, Sha256};
use tendermint::v0_34::abci::{
    request::{
        ApplySnapshotChunk,
        BeginBlock,
        CheckTx,
        DeliverTx,
        EndBlock,
        InitChain,
        LoadSnapshotChunk,
        OfferSnapshot,
        Query,
    },
    ConsensusRequest,
    InfoRequest,
    MempoolRequest,
    SnapshotRequest,
};
use tracing::{error_span, Span};

pub trait RequestExt {
    fn create_span(&self) -> Span;
}

impl RequestExt for ConsensusRequest {
    fn create_span(&self) -> Span {
        let p = error_span!("abci");

        match self {
            Self::InitChain(InitChain { chain_id, .. }) => {
                error_span!(parent: &p, "InitChain", ?chain_id)
            },
            Self::BeginBlock(BeginBlock { header, .. }) => {
                error_span!(parent: &p, "BeginBlock", height = ?header.height.value())
            },
            Self::DeliverTx(DeliverTx { tx }) => {
                error_span!(parent: &p, "DeliverTx", tx_id = ?hex::encode(Sha256::digest(tx.as_ref())))
            },
            Self::EndBlock(EndBlock { height }) => {
                error_span!(parent: &p, "EndBlock", ?height)
            },
            Self::Commit => {
                error_span!(parent: &p, "Commit")
            },
        }
    }
}

impl RequestExt for InfoRequest {
    fn create_span(&self) -> Span {
        let p = error_span!("abci");

        match self {
            Self::Info(_) => error_span!(parent: &p, "Info"),
            Self::Query(Query {
                path,
                height,
                prove,
                ..
            }) => {
                error_span!(parent: &p, "Query", ?path, ?height, prove)
            },
            Self::Echo(_) => error_span!(parent: &p, "Echo"),
            Self::SetOption(_) => todo!("not implemented"),
        }
    }
}

impl RequestExt for MempoolRequest {
    fn create_span(&self) -> Span {
        let p = error_span!("abci");

        match self {
            Self::CheckTx(CheckTx { kind, tx }) => {
                error_span!(parent: &p, "CheckTx", ?kind, tx_id = ?hex::encode(Sha256::digest(tx.as_ref())))
            },
        }
    }
}

impl RequestExt for SnapshotRequest {
    fn create_span(&self) -> Span {
        let p = error_span!("abci");

        match self {
            Self::ListSnapshots => {
                error_span!(parent: &p, "ListSnapshots")
            },
            Self::OfferSnapshot(OfferSnapshot { snapshot, app_hash }) => {
                error_span!(parent: &p, "OfferSnapshot", app_hash = ?app_hash, height = snapshot.height.value(), chunks = snapshot.chunks)
            },
            Self::LoadSnapshotChunk(LoadSnapshotChunk { height, chunk, .. }) => {
                error_span!(
                    parent: &p,
                    "LoadSnapshotChunk",
                    height = height.value(),
                    chunk = chunk
                )
            },
            Self::ApplySnapshotChunk(ApplySnapshotChunk { index, sender, .. }) => {
                error_span!(
                    parent: &p,
                    "ApplySnapshotChunk",
                    index = index,
                    sender = sender
                )
            },
        }
    }
}
