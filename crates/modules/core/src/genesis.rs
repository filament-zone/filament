use anyhow::Result;
use sov_modules_api::{Spec, WorkingSet};

use crate::{Campaign, Core, Indexer};

#[derive(Debug, Clone, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[cfg_attr(
    feature = "native",
    derive(schemars::JsonSchema),
    schemars(bound = "S: ::sov_modules_api::Spec", rename = "CoreConfig")
)]
#[serde(bound = "S::Address: serde::Serialize + serde::de::DeserializeOwned")]
pub struct CoreConfig<S: Spec> {
    pub admin: S::Address,

    pub campaigns: Vec<Campaign<S>>,
    pub delegates: Vec<S::Address>,
    pub indexers: Vec<Indexer<S>>,
}

impl<S: Spec> Core<S> {
    pub(crate) fn init_module(
        &self,
        config: &<Self as sov_modules_api::Module>::Config,
        working_set: &mut WorkingSet<S>,
    ) -> Result<()> {
        tracing::info!(?config, "starting core genesis");

        self.admin.set(&config.admin, working_set);

        for delegate in config.delegates.iter() {
            self.delegates.push(delegate, working_set);
        }

        let mut id = 0;
        for campaign in config.campaigns.iter() {
            self.campaigns.set(&id, campaign, working_set);
            id += 1;
        }
        self.next_campaign_id.set(&id, working_set);

        for Indexer { addr, alias } in config.indexers.iter() {
            self.indexers.push(addr, working_set);
            self.indexer_aliases.set(addr, alias, working_set);
        }

        tracing::info!("completed core genesis");

        Ok(())
    }
}
