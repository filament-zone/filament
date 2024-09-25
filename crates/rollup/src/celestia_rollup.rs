#![deny(missing_docs)]
//! StarterRollup provides a minimal self-contained rollup implementation

use async_trait::async_trait;
use backon::ExponentialBuilder;
use filament_hub_stf::Runtime;
use sov_celestia_adapter::{
    types::Namespace,
    verifier::{CelestiaSpec, CelestiaVerifier, RollupParams},
    CelestiaService,
};
use sov_db::{ledger_db::LedgerDb, storage_manager::NativeStorageManager};
use sov_kernels::basic::BasicKernel;
use sov_mock_zkvm::{MockCodeCommitment, MockZkVerifier, MockZkvm};
use sov_modules_api::{
    capabilities::Kernel,
    default_spec::DefaultSpec,
    CryptoSpec,
    OperatingMode,
    SovApiProofSerializer,
    Spec,
};
use sov_modules_rollup_blueprint::{
    pluggable_traits::PluggableSpec,
    FullNodeBlueprint,
    RollupBlueprint,
};
use sov_modules_stf_blueprint::{RuntimeEndpoints, StfBlueprint};
use sov_risc0_adapter::{host::Risc0Host, Risc0Verifier};
use sov_rollup_interface::{
    execution_mode::{ExecutionMode, Native, Zk},
    node::da::{DaService, DaServiceWithRetries},
    zk::{aggregated_proof::CodeCommitment, Zkvm},
};
use sov_sequencer::{FairBatchBuilderConfig, SequencerDb};
use sov_state::{DefaultStorageSpec, ProverStorage, Storage, ZkStorage};
use sov_stf_runner::{
    processes::{ParallelProverService, ProverService, RollupProverConfig},
    RollupConfig,
};
use tokio::sync::watch;

/// The rollup stores its data in the namespace "sov-test-b" on Celestia.
/// You can change this constant to point your rollup at a different namespace.
const ROLLUP_BATCH_NAMESPACE: Namespace = Namespace::const_v0(*b"sov-test-b");

/// The rollup stores the zk proofs in the namespace "sov-test-p" on Celestia.
const ROLLUP_PROOF_NAMESPACE: Namespace = Namespace::const_v0(*b"sov-test-p");

/// Rollup with [`CelestiaDaService`].
#[derive(Default)]
pub struct CelestiaRollup<M> {
    phantom: std::marker::PhantomData<M>,
}

/// This is the place, where all the rollup components come together, and
/// they can be easily swapped with alternative implementations as needed.
#[async_trait]
impl<M: ExecutionMode> RollupBlueprint<M> for CelestiaRollup<M>
where
    DefaultSpec<Risc0Verifier, MockZkVerifier, M>: PluggableSpec,
{
    type DaSpec = CelestiaSpec;
    type Kernel = BasicKernel<Self::Spec, Self::DaSpec>;
    type Runtime = Runtime<Self::Spec, Self::DaSpec>;
    type Spec = DefaultSpec<Risc0Verifier, MockZkVerifier, M>;
}

#[async_trait]
impl FullNodeBlueprint<Native> for CelestiaRollup<Native> {
    type DaService = DaServiceWithRetries<CelestiaService>;
    /// Inner Zkvm representing the rollup circuit
    type InnerZkvmHost = Risc0Host<'static>;
    /// Outer Zkvm representing the circuit verifier for recursion
    type OuterZkvmHost = MockZkvm;
    type ProofSerializer = SovApiProofSerializer<Self::Spec>;
    type ProverService = ParallelProverService<
        <Self::Spec as Spec>::Address,
        <<Self::Spec as Spec>::Storage as Storage>::Root,
        <<Self::Spec as Spec>::Storage as Storage>::Witness,
        Self::DaService,
        Self::InnerZkvmHost,
        Self::OuterZkvmHost,
        StfBlueprint<
            <CelestiaRollup<Zk> as RollupBlueprint<Zk>>::Spec,
            Self::DaSpec,
            <CelestiaRollup<Zk> as RollupBlueprint<Zk>>::Runtime,
            <CelestiaRollup<Zk> as RollupBlueprint<Zk>>::Kernel,
        >,
    >;
    type StorageManager = NativeStorageManager<
        CelestiaSpec,
        ProverStorage<DefaultStorageSpec<<<Self::Spec as Spec>::CryptoSpec as CryptoSpec>::Hasher>>,
    >;

    fn get_operating_mode(
        genesis: &<Self::Kernel as Kernel<<Self::Spec as Spec>::Storage>>::GenesisConfig,
    ) -> OperatingMode {
        genesis.chain_state.operating_mode
    }

    fn create_outer_code_commitment(
        &self,
    ) -> <<Self::ProverService as ProverService>::Verifier as Zkvm>::CodeCommitment {
        MockCodeCommitment::default()
    }

    fn create_endpoints(
        &self,
        storage: watch::Receiver<<Self::Spec as Spec>::Storage>,
        ledger_db: &LedgerDb,
        sequencer_db: &SequencerDb,
        da_service: &Self::DaService,
        rollup_config: &RollupConfig<
            <Self::Spec as Spec>::Address,
            <Self::DaService as DaService>::Config,
            FairBatchBuilderConfig<Self::DaSpec>,
        >,
    ) -> anyhow::Result<RuntimeEndpoints> {
        sov_modules_rollup_blueprint::register_endpoints::<Self, _>(
            storage.clone(),
            ledger_db,
            sequencer_db,
            da_service,
            &rollup_config.sequencer,
        )
    }

    async fn create_da_service(
        &self,
        rollup_config: &RollupConfig<
            <Self::Spec as Spec>::Address,
            <Self::DaService as DaService>::Config,
            FairBatchBuilderConfig<Self::DaSpec>,
        >,
    ) -> Self::DaService {
        DaServiceWithRetries::with_exponential_backoff(
            CelestiaService::new(
                rollup_config.da.clone(),
                RollupParams {
                    rollup_batch_namespace: ROLLUP_BATCH_NAMESPACE,
                    rollup_proof_namespace: ROLLUP_PROOF_NAMESPACE,
                },
            )
            .await,
            // NOTE: Current exponential backoff policy defaults:
            // jitter: false, factor: 2, min_delay: 1s, max_delay: 60s, max_times: 3,
            ExponentialBuilder::default(),
        )
    }

    async fn create_prover_service(
        &self,
        prover_config: RollupProverConfig,
        rollup_config: &RollupConfig<
            <Self::Spec as Spec>::Address,
            <Self::DaService as DaService>::Config,
            FairBatchBuilderConfig<Self::DaSpec>,
        >,
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
            rollup_config.proof_manager.prover_address,
        )
    }

    fn create_storage_manager(
        &self,
        rollup_config: &RollupConfig<
            <Self::Spec as Spec>::Address,
            <Self::DaService as DaService>::Config,
            FairBatchBuilderConfig<Self::DaSpec>,
        >,
    ) -> anyhow::Result<Self::StorageManager> {
        NativeStorageManager::new(&rollup_config.storage.path)
    }
}

impl sov_modules_rollup_blueprint::WalletBlueprint<Native> for CelestiaRollup<Native> {}
