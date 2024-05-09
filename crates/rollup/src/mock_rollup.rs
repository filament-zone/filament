use async_trait::async_trait;
use filament_hub_stf::{authentication::ModAuth, genesis::StorageConfig, runtime::Runtime};
use sov_db::ledger_db::LedgerDb;
use sov_kernels::basic::BasicKernel;
use sov_mock_da::{MockDaConfig, MockDaService, MockDaSpec};
use sov_mock_zkvm::{MockCodeCommitment, MockZkvm};
use sov_modules_api::{
    default_spec::{DefaultSpec, ZkDefaultSpec},
    Spec,
    Zkvm,
};
use sov_modules_rollup_blueprint::RollupBlueprint;
use sov_modules_stf_blueprint::StfBlueprint;
use sov_prover_storage_manager::ProverStorageManager;
use sov_risc0_adapter::host::Risc0Host;
use sov_rollup_interface::zk::{aggregated_proof::CodeCommitment, ZkvmGuest, ZkvmHost};
use sov_sequencer::SequencerDb;
use sov_state::{DefaultStorageSpec, Storage, ZkStorage};
use sov_stf_runner::{ParallelProverService, ProverService, RollupConfig, RollupProverConfig};
use tokio::sync::watch;

/// Rollup with MockDa
pub struct MockDemoRollup {}

#[async_trait]
impl RollupBlueprint for MockDemoRollup {
    type DaConfig = MockDaConfig;
    type DaService = MockDaService;
    type DaSpec = MockDaSpec;
    type InnerZkvmHost = Risc0Host<'static>;
    type NativeKernel = BasicKernel<Self::NativeSpec, Self::DaSpec>;
    type NativeRuntime = Runtime<Self::NativeSpec, Self::DaSpec>;
    type NativeSpec = DefaultSpec<
        <<Self::InnerZkvmHost as ZkvmHost>::Guest as ZkvmGuest>::Verifier,
        <<Self::OuterZkvmHost as ZkvmHost>::Guest as ZkvmGuest>::Verifier,
    >;
    type OuterZkvmHost = MockZkvm;
    type ProverService = ParallelProverService<
        <<Self::NativeSpec as Spec>::Storage as Storage>::Root,
        <<Self::NativeSpec as Spec>::Storage as Storage>::Witness,
        Self::DaService,
        Self::InnerZkvmHost,
        Self::OuterZkvmHost,
        StfBlueprint<Self::ZkSpec, Self::DaSpec, Self::ZkRuntime, Self::ZkKernel>,
    >;
    type StorageManager = ProverStorageManager<MockDaSpec, DefaultStorageSpec>;
    type ZkKernel = BasicKernel<Self::ZkSpec, Self::DaSpec>;
    type ZkRuntime = Runtime<Self::ZkSpec, Self::DaSpec>;
    type ZkSpec = ZkDefaultSpec<
        <<Self::InnerZkvmHost as ZkvmHost>::Guest as ZkvmGuest>::Verifier,
        <<Self::OuterZkvmHost as ZkvmHost>::Guest as ZkvmGuest>::Verifier,
    >;

    fn create_outer_code_commitment(
        &self,
    ) -> <<Self::ProverService as ProverService>::Verifier as Zkvm>::CodeCommitment {
        MockCodeCommitment::default()
    }

    fn create_endpoints(
        &self,
        storage: watch::Receiver<<Self::NativeSpec as Spec>::Storage>,
        ledger_db: &LedgerDb,
        sequencer_db: &SequencerDb,
        da_service: &Self::DaService,
        rollup_config: &RollupConfig<Self::DaConfig>,
    ) -> Result<(jsonrpsee::RpcModule<()>, axum::Router<()>), anyhow::Error> {
        #[allow(unused_mut)]
        let (mut rpc_methods, axum_router) = sov_modules_rollup_blueprint::register_endpoints::<
            Self,
            ModAuth<Self::NativeSpec, Self::DaSpec>,
        >(
            storage.clone(),
            ledger_db,
            sequencer_db,
            da_service,
            rollup_config.da.sender_address,
        )?;

        Ok((rpc_methods, axum_router))
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
        _rollup_config: &RollupConfig<Self::DaConfig>,
        _da_service: &Self::DaService,
    ) -> Self::ProverService {
        let inner_vm = Risc0Host::new(filament_prover_risc0::MOCK_DA_ELF);
        let outer_vm = MockZkvm::new_non_blocking();
        let zk_stf = StfBlueprint::new();
        let zk_storage = ZkStorage::new();
        let da_verifier = Default::default();

        ParallelProverService::new_with_default_workers(
            inner_vm,
            outer_vm,
            zk_stf,
            da_verifier,
            prover_config,
            zk_storage,
            CodeCommitment::default(),
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
