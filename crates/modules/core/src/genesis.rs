use std::collections::HashMap;

use anyhow::Result;
use sov_modules_api::{GenesisState, Spec};

use crate::{delegate::Delegate, Campaign, Core, Indexer, Power, Relayer};

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
    pub delegates: Vec<Delegate<S>>,
    pub eth_addresses: HashMap<S::Address, String>,
    pub indexers: Vec<Indexer<S>>,
    pub powers: HashMap<S::Address, Power>,
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

        let mut id = 0;
        for campaign in config.campaigns.iter() {
            self.campaigns.set(&id, campaign, state)?;
            id += 1;
        }
        self.next_campaign_id.set(&id, state)?;

        for delegate in config.delegates.iter() {
            self.delegates.push(&delegate.address, state)?;
        }

        for (addr, eth_addr) in &config.eth_addresses {
            self.eth_addresses.set(addr, eth_addr, state)?;
        }

        for Indexer { addr, alias } in config.indexers.iter() {
            self.indexers.push(addr, state)?;
            self.indexer_aliases.set(addr, alias, state)?;
        }

        for relayer in config.relayers.iter() {
            self.relayers.push(relayer, state)?;
        }

        let mut index = config
            .powers
            .iter()
            .map(|(addr, power)| (addr.clone(), *power))
            .collect::<Vec<_>>();
        index.sort_unstable_by(|a, b| b.1.cmp(&a.1));
        for (addr, power) in &index {
            self.powers.set(addr, power, state)?;
        }
        self.powers_index.set_all(index, state)?;

        tracing::info!("completed core genesis");

        Ok(())
    }
}
