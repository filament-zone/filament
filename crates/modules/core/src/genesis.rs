use anyhow::Result;
use sov_modules_api::{GenesisState, Spec};

use crate::{Campaign, Core, Indexer, Relayer};

#[cfg_attr(
    feature = "native",
    derive(schemars::JsonSchema),
    schemars(bound = "S: ::sov_modules_api::Spec", rename = "CoreConfig")
)]
#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(bound = "S::Address: serde::Serialize + serde::de::DeserializeOwned")]
pub struct CoreConfig<S: Spec> {
    pub admin: S::Address,

    pub campaigns: Vec<Campaign<S>>,
    pub delegates: Vec<S::Address>,
    pub indexers: Vec<Indexer<S>>,
    pub relayers: Vec<Relayer<S>>,
}

impl<S: Spec> Core<S> {
    pub(crate) fn init_module(
        &self,
        config: &<Self as sov_modules_api::Module>::Config,
        state: &mut impl GenesisState<S>,
    ) -> Result<()> {
        tracing::info!(?config, "starting core genesis");

        self.admin.set(&config.admin, state)?;

        for delegate in config.delegates.iter() {
            self.delegates.push(delegate, state)?;
        }

        let mut id = 0;
        for campaign in config.campaigns.iter() {
            self.campaigns.set(&id, campaign, state)?;
            id += 1;
        }
        self.next_campaign_id.set(&id, state)?;

        for Indexer { addr, alias } in config.indexers.iter() {
            self.indexers.push(addr, state)?;
            self.indexer_aliases.set(addr, alias, state)?;
        }

        for relayer in config.relayers.iter() {
            self.relayers.push(relayer, state)?;
        }

        tracing::info!("completed core genesis");

        Ok(())
    }
}
