use anyhow::Result;
use serde::{Deserialize, Serialize};
use sov_modules_api::prelude::*;
use sov_modules_api::{Context, WorkingSet};

use crate::OutpostRegistry;

/// Config for the OutpostsRegistry module.
/// Sets admin.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct OutpostRegistryConfig<C>
where
    C: Context,
{
    /// Admin of the OutpostRegistry module.
    pub admin: C::Address,
}

impl<C> OutpostRegistry<C>
where
    C: Context,
{
    pub(crate) fn init_module(
        &self,
        config: &<Self as sov_modules_api::Module>::Config,
        working_set: &mut WorkingSet<C>,
    ) -> Result<()> {
        self.admin.set(&config.admin, working_set);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use sov_modules_api::default_context::DefaultContext;
    use sov_modules_api::utils::generate_address;
    use sov_modules_api::Spec;

    use super::OutpostRegistryConfig;

    #[test]
    fn test_config_serialization() {
        let address: <DefaultContext as Spec>::Address =
            generate_address::<DefaultContext>("admin");

        let config = OutpostRegistryConfig::<DefaultContext> { admin: address };

        let data = r#"
        {
            "admin":"sov1335hded4gyzpt00fpz75mms4m7ck02wgw07yhw9grahj4dzg4yvqk63pml"
        }"#;

        let parsed_config: OutpostRegistryConfig<DefaultContext> =
            serde_json::from_str(data).unwrap();
        assert_eq!(config, parsed_config)
    }
}
