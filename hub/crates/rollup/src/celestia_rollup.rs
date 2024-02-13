#![deny(missing_docs)]
//! StarterRollup provides a minimal self-contained rollup implementation

use async_trait::async_trait;
use hub_stf::Runtime;
use sov_celestia_adapter::{
    types::Namespace,
    verifier::{CelestiaSpec, CelestiaVerifier, RollupParams},
    CelestiaConfig,
    CelestiaService,
};
use sov_modules_api::{
    default_context::{DefaultContext, ZkDefaultContext},
    Address,
    Spec,
};
use sov_modules_rollup_blueprint::RollupBlueprint;
use sov_modules_stf_blueprint::{kernels::basic::BasicKernel, StfBlueprint};
use sov_prover_storage_manager::ProverStorageManager;
use sov_risc0_adapter::host::Risc0Host;
use sov_rollup_interface::zk::ZkvmHost;
use sov_state::{config::Config as StorageConfig, DefaultStorageSpec, Storage, ZkStorage};
use sov_stf_runner::{ParallelProverService, RollupConfig, RollupProverConfig};

/// The namespace for the rollup on Celestia.
const ROLLUP_NAMESPACE: Namespace = Namespace::const_v0(*b"filahubtia");

/// The rollup stores the zk proofs in the namespace b"sov-test-p" on Celestia.
const ROLLUP_PROOF_NAMESPACE: Namespace = Namespace::const_v0(*b"filaproofs");

/// Rollup with [`CelestiaDaService`].
pub struct CelestiaRollup {}

/// This is the place, where all the rollup components come together and
/// they can be easily swapped with alternative implementations as needed.
#[async_trait]
impl RollupBlueprint for CelestiaRollup {
    type DaConfig = CelestiaConfig;
    type DaService = CelestiaService;
    type DaSpec = CelestiaSpec;
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
    type StorageManager = ProverStorageManager<CelestiaSpec, DefaultStorageSpec>;
    type Vm = Risc0Host<'static>;
    type ZkContext = ZkDefaultContext;
    type ZkKernel = BasicKernel<Self::ZkContext, Self::DaSpec>;
    type ZkRuntime = Runtime<Self::ZkContext, Self::DaSpec>;

    fn create_rpc_methods(
        &self,
        storage: &<Self::NativeContext as sov_modules_api::Spec>::Storage,
        ledger_db: &sov_db::ledger_db::LedgerDB,
        da_service: &Self::DaService,
    ) -> Result<jsonrpsee::RpcModule<()>, anyhow::Error> {
        // TODO set the sequencer address
        let sequencer = Address::new([0; 32]);

        #[allow(unused_mut)]
        let mut rpc_methods = sov_modules_rollup_blueprint::register_rpc::<
            Self::NativeRuntime,
            Self::NativeContext,
            Self::DaService,
        >(storage, ledger_db, da_service, sequencer)?;

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
        CelestiaService::new(
            rollup_config.da.clone(),
            RollupParams {
                rollup_batch_namespace: ROLLUP_NAMESPACE,
                rollup_proof_namespace: ROLLUP_PROOF_NAMESPACE,
            },
        )
        .await
    }

    async fn create_prover_service(
        &self,
        prover_config: RollupProverConfig,
        rollup_config: &RollupConfig<Self::DaConfig>,
        _da_service: &Self::DaService,
    ) -> Self::ProverService {
        let vm = Risc0Host::new(hub_prover::ROLLUP_ELF);
        let zk_stf = StfBlueprint::new();
        let zk_storage = ZkStorage::new();

        let da_verifier = CelestiaVerifier {
            rollup_namespace: ROLLUP_NAMESPACE,
        };

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
        rollup_config: &sov_stf_runner::RollupConfig<Self::DaConfig>,
    ) -> Result<Self::StorageManager, anyhow::Error> {
        let storage_config = StorageConfig {
            path: rollup_config.storage.path.clone(),
        };
        ProverStorageManager::new(storage_config)
    }
}
impl sov_modules_rollup_blueprint::WalletBlueprint for CelestiaRollup {}