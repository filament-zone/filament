use anyhow::Result;
use sov_modules_api::{Spec, WorkingSet};

use crate::Campaigns;

impl<S: Spec> Campaigns<S> {
    pub(crate) fn init_module(
        &self,
        config: &<Self as sov_modules_api::Module>::Config,
        working_set: &mut WorkingSet<S>,
    ) -> Result<()> {
        tracing::info!(?config, "starting campaigns genesis");

        let mut id = 1;

        for campaign in config.campaigns.iter() {
            self.campaigns.set(&id, campaign, working_set);
            id += 1;
        }

        self.next_id.set(&id, working_set);

        tracing::info!("completed campaigns genesis");

        Ok(())
    }
}
