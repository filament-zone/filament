use async_trait::async_trait;
use filament_hub_stf::{authentication::ModAuth, genesis::StorageConfig, runtime::Runtime};
use sov_celestia_adapter::{
    verifier::{CelestiaSpec, CelestiaVerifier, RollupParams},
    CelestiaConfig,
    CelestiaService,
};
use sov_db::ledger_db::LedgerDb;
use sov_kernels::basic::BasicKernel;
use sov_mock_zkvm::{MockCodeCommitment, MockZkvm};
use sov_modules_api::{
    default_spec::{DefaultSpec, ZkDefaultSpec},
    Spec,
};
use sov_modules_rollup_blueprint::{RollupBlueprint, WalletBlueprint};
use sov_modules_stf_blueprint::StfBlueprint;
use sov_prover_storage_manager::ProverStorageManager;
use sov_risc0_adapter::host::Risc0Host;
use sov_rollup_interface::zk::{aggregated_proof::CodeCommitment, Zkvm, ZkvmGuest, ZkvmHost};
use sov_sequencer::SequencerDb;
use sov_state::{DefaultStorageSpec, Storage, ZkStorage};
use sov_stf_runner::{ParallelProverService, ProverService, RollupConfig, RollupProverConfig};
use tokio::sync::watch;

use crate::{ROLLUP_BATCH_NAMESPACE, ROLLUP_PROOF_NAMESPACE};

/// Rollup with CelestiaDa
pub struct CelestiaDemoRollup {}

#[async_trait]
impl RollupBlueprint for CelestiaDemoRollup {
    type DaConfig = CelestiaConfig;
    type DaService = CelestiaService;
    type DaSpec = CelestiaSpec;
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
    type StorageManager = ProverStorageManager<CelestiaSpec, DefaultStorageSpec>;
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
        let sequencer = rollup_config.da.own_celestia_address.clone();

        #[allow(unused_mut)]
        let (mut rpc_methods, axum_router) = sov_modules_rollup_blueprint::register_endpoints::<
            Self,
            ModAuth<Self::NativeSpec, Self::DaSpec>,
        >(
            storage.clone(),
            ledger_db,
            sequencer_db,
            da_service,
            sequencer,
        )?;

        Ok((rpc_methods, axum_router))
    }

    async fn create_da_service(
        &self,
        rollup_config: &RollupConfig<Self::DaConfig>,
    ) -> Self::DaService {
        CelestiaService::new(
            rollup_config.da.clone(),
            RollupParams {
                rollup_batch_namespace: ROLLUP_BATCH_NAMESPACE,
                rollup_proof_namespace: ROLLUP_PROOF_NAMESPACE,
            },
        )
        .await
    }

    async fn create_prover_service(
        &self,
        prover_config: RollupProverConfig,
        _rollup_config: &RollupConfig<Self::DaConfig>,
        _da_service: &Self::DaService,
    ) -> Self::ProverService {
        let inner_vm = Risc0Host::new(filament_prover_risc0::ROLLUP_ELF);
        let outer_vm = MockZkvm::new_non_blocking();

        let zk_stf = StfBlueprint::new();
        let zk_storage = ZkStorage::new();

        let da_verifier = CelestiaVerifier {
            rollup_batch_namespace: ROLLUP_BATCH_NAMESPACE,
            rollup_proof_namespace: ROLLUP_PROOF_NAMESPACE,
        };

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
    ) -> Result<Self::StorageManager, anyhow::Error> {
        let storage_config = StorageConfig {
            path: rollup_config.storage.path.clone(),
        };
        ProverStorageManager::new(storage_config)
    }
}

impl WalletBlueprint for CelestiaDemoRollup {}
