use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use hub_stf::{genesis_config::StorageConfig, runtime::Runtime};
use sov_db::ledger_db::LedgerDB;
use sov_mock_da::{MockDaConfig, MockDaService, MockDaSpec};
use sov_modules_api::{
    default_context::{DefaultContext, ZkDefaultContext},
    Address,
    Spec,
};
use sov_modules_rollup_blueprint::{RollupBlueprint, WalletBlueprint};
use sov_modules_stf_blueprint::{kernels::basic::BasicKernel, StfBlueprint};
use sov_prover_storage_manager::ProverStorageManager;
use sov_risc0_adapter::host::Risc0Host;
use sov_rollup_interface::zk::ZkvmHost;
use sov_state::{DefaultStorageSpec, Storage, ZkStorage};
use sov_stf_runner::{ParallelProverService, RollupConfig, RollupProverConfig};

/// Rollup with MockDa
pub struct MockRollup {}

impl WalletBlueprint for MockRollup {}

#[async_trait]
impl RollupBlueprint for MockRollup {
    type DaConfig = MockDaConfig;
    type DaService = MockDaService;
    type DaSpec = MockDaSpec;
    type NativeContext = DefaultContext;
    type NativeKernel = BasicKernel<Self::NativeContext, Self::DaSpec>;
    type NativeRuntime = Runtime<Self::NativeContext, Self::DaSpec>;
    type ProverService = ParallelProverService<
        <<Self::NativeContext as Spec>::Storage as Storage>::Root,
        <<Self::NativeContext as Spec>::Storage as Storage>::Witness,
        Self::DaService,
        Self::Vm,
        StfBlueprint<
            Self::ZkContext,
            Self::DaSpec,
            <Self::Vm as ZkvmHost>::Guest,
            Self::ZkRuntime,
            Self::ZkKernel,
        >,
    >;
    type StorageManager = ProverStorageManager<MockDaSpec, DefaultStorageSpec>;
    type Vm = Risc0Host<'static>;
    type ZkContext = ZkDefaultContext;
    type ZkKernel = BasicKernel<Self::ZkContext, Self::DaSpec>;
    type ZkRuntime = Runtime<Self::ZkContext, Self::DaSpec>;

    fn create_rpc_methods(
        &self,
        storage: Arc<RwLock<<Self::NativeContext as Spec>::Storage>>,
        ledger_db: &LedgerDB,
        da_service: &Self::DaService,
    ) -> Result<jsonrpsee::RpcModule<()>, anyhow::Error> {
        // TODO set the sequencer address
        let sequencer = Address::new([0; 32]);

        #[allow(unused_mut)]
        let mut rpc_methods = sov_modules_rollup_blueprint::register_rpc::<
            Self::NativeRuntime,
            Self::NativeContext,
            Self::DaService,
        >(storage.clone(), ledger_db, da_service, sequencer)?;

        #[cfg(feature = "experimental")]
        crate::eth::register_ethereum::<Self::DaService>(
            da_service.clone(),
            storage.clone(),
            &mut rpc_methods,
        )?;

        Ok(rpc_methods)
    }

    async fn create_da_service(
        &self,
        rollup_config: &RollupConfig<Self::DaConfig>,
    ) -> Self::DaService {
        MockDaService::from_config(rollup_config.da.clone())
    }

    async fn create_prover_service(
        &self,
        prover_config: RollupProverConfig,
        rollup_config: &RollupConfig<Self::DaConfig>,
        _da_service: &Self::DaService,
    ) -> Self::ProverService {
        let vm = Risc0Host::new(hub_prover::MOCK_DA_ELF);
        let zk_stf = StfBlueprint::new();
        let zk_storage = ZkStorage::new();
        let da_verifier = Default::default();

        ParallelProverService::new_with_default_workers(
            vm,
            zk_stf,
            da_verifier,
            prover_config,
            zk_storage,
            rollup_config.prover_service,
        )
    }

    fn create_storage_manager(
        &self,
        rollup_config: &RollupConfig<Self::DaConfig>,
    ) -> anyhow::Result<Self::StorageManager> {
        let storage_config = StorageConfig {
            path: rollup_config.storage.path.clone(),
        };
        ProverStorageManager::new(storage_config)
    }
}
