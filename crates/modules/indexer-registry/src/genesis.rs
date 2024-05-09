use anyhow::Result;
use sov_modules_api::{Spec, WorkingSet};

use crate::IndexerRegistry;

impl<S: Spec> IndexerRegistry<S> {
    pub(crate) fn init_module(
        &self,
        config: &<Self as sov_modules_api::Module>::Config,
        working_set: &mut WorkingSet<S>,
    ) -> Result<()> {
        self.admin.set(&config.admin, working_set);

        for (addr, alias) in config.indexers.iter() {
            self.indexers.push(addr, working_set);
            self.aliases.set(addr, alias, working_set);
        }

        Ok(())
    }
}
