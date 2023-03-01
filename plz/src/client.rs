use pulzaar_app::Query;
use serde::{de::DeserializeOwned, Serialize};
use tendermint::block::Height;
use tendermint_rpc::{endpoint::broadcast::tx_commit, Client as _, HttpClient};
use tokio::runtime::Runtime;

pub struct Client {
    client: HttpClient,
    runtime: Runtime,
}

impl Client {
    pub fn new(node_uri: &str) -> eyre::Result<Self> {
        Ok(Self {
            client: HttpClient::new(node_uri)?,
            runtime: Runtime::new()?,
        })
    }

    pub fn broadcast_tx_commit<T>(&self, tx: T) -> eyre::Result<tx_commit::Response>
    where
        T: Into<Vec<u8>> + Send,
    {
        let broadcast = self.client.broadcast_tx_commit(tx);
        Ok(self.runtime.block_on(broadcast)?)
    }

    pub fn query<R>(&self, height: Option<u64>, query: impl Query + Serialize) -> eyre::Result<R>
    where
        R: DeserializeOwned,
    {
        let height = height.map(Height::try_from).transpose()?;

        let path = Some(query.prefix().to_string());
        let data = pulzaar_encoding::to_bytes(&query)?;
        let query = self.client.abci_query(path, data, height, false);
        let res = self.runtime.block_on(query)?;

        if res.code.is_err() {
            eyre::bail!("ABCI account query error {:?}", res);
        }

        let r = pulzaar_encoding::from_bytes::<R>(&res.value)?;

        Ok(r)
    }
}
