#![deny(missing_docs)]
//! StarterRollup provides a minimal self-contained rollup implementation

use anyhow::Error;
use async_trait::async_trait;
use filament_hub_stf::Runtime;
use sov_attester_incentives::BondingProofServiceImpl;
use sov_db::{ledger_db::LedgerDb, storage_manager::NativeStorageManager};
use sov_kernels::basic::BasicKernel;
use sov_mock_da::{storable::service::StorableMockDaService, MockDaSpec};
use sov_mock_zkvm::{MockCodeCommitment, MockZkVerifier, MockZkvm};
use sov_modules_api::{
    capabilities::Kernel,
    default_spec::DefaultSpec,
    higher_kinded_types::Generic,
    CryptoSpec,
    OperatingMode,
    SovApiProofSerializer,
    Spec,
    Zkvm,
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
    node::da::DaServiceWithRetries,
    zk::aggregated_proof::CodeCommitment,
};
use sov_sequencer::SequencerDb;
use sov_state::{DefaultStorageSpec, ProverStorage, Storage, ZkStorage};
use sov_stf_runner::{
    processes::{ParallelProverService, ProverService, RollupProverConfig},
    RollupConfig,
};
use tokio::sync::watch;

/// Rollup with [`MockDaService`].
#[derive(Default)]
pub struct MockRollup<M> {
    phantom: std::marker::PhantomData<M>,
}

/// This is the place, where all the rollup components come together, and
/// they can be easily swapped with alternative implementations as needed.
#[async_trait]
impl<M: ExecutionMode> RollupBlueprint<M> for MockRollup<M>
where
    DefaultSpec<Risc0Verifier, MockZkVerifier, M>: PluggableSpec,
{
    type DaSpec = MockDaSpec;
    type Kernel = BasicKernel<Self::Spec, Self::DaSpec>;
    type Runtime = Runtime<Self::Spec, Self::DaSpec>;
    type Spec = DefaultSpec<Risc0Verifier, MockZkVerifier, M>;
}

#[async_trait]
impl FullNodeBlueprint<Native> for MockRollup<Native> {
    type BondingProofService = BondingProofServiceImpl<Self::Spec, Self::DaSpec>;
    type DaService = DaServiceWithRetries<StorableMockDaService>;
    /// Inner Zkvm representing the rollup circuit
    type InnerZkvmHost = Risc0Host<'static>;
    /// Outer Zkvm representing the circuit verifier for recursion
    type OuterZkvmHost = MockZkvm;
    type ProofSerializer = SovApiProofSerializer<Self::Spec>;
    /// Prover service.
    type ProverService = ParallelProverService<
        <Self::Spec as Spec>::Address,
        <<Self::Spec as Spec>::Storage as Storage>::Root,
        <<Self::Spec as Spec>::Storage as Storage>::Witness,
        Self::DaService,
        Self::InnerZkvmHost,
        Self::OuterZkvmHost,
        StfBlueprint<
            <Self::Spec as Generic>::With<Zk>,
            Self::DaSpec,
            <MockRollup<Zk> as RollupBlueprint<Zk>>::Runtime,
            <MockRollup<Zk> as RollupBlueprint<Zk>>::Kernel,
        >,
    >;
    /// Manager for the native storage lifecycle.
    type StorageManager = NativeStorageManager<
        MockDaSpec,
        ProverStorage<DefaultStorageSpec<<<Self::Spec as Spec>::CryptoSpec as CryptoSpec>::Hasher>>,
    >;

    fn create_bonding_proof_service(
        &self,
        attester_address: <Self::Spec as Spec>::Address,
        storage: tokio::sync::watch::Receiver<<Self::Spec as Spec>::Storage>,
    ) -> Self::BondingProofService {
        let runtime = Runtime::<Self::Spec, Self::DaSpec>::default();
        BondingProofServiceImpl::new(attester_address, runtime.attester_incentives, storage)
    }

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

    async fn create_endpoints(
        &self,
        storage: watch::Receiver<<Self::Spec as Spec>::Storage>,
        ledger_db: &LedgerDb,
        sequencer_db: &SequencerDb,
        da_service: &Self::DaService,
        rollup_config: &RollupConfig<<Self::Spec as Spec>::Address, Self::DaService>,
    ) -> Result<RuntimeEndpoints, Error> {
        sov_modules_rollup_blueprint::register_endpoints::<Self, Native>(
            storage.clone(),
            ledger_db,
            sequencer_db,
            da_service,
            &rollup_config.sequencer,
        )
        .await
    }

    async fn create_da_service(
        &self,
        rollup_config: &RollupConfig<<Self::Spec as Spec>::Address, Self::DaService>,
    ) -> Self::DaService {
        DaServiceWithRetries::new_fast(
            StorableMockDaService::from_config(rollup_config.da.clone()).await,
        )
    }

    async fn create_prover_service(
        &self,
        prover_config: RollupProverConfig,
        rollup_config: &RollupConfig<<Self::Spec as Spec>::Address, Self::DaService>,
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
            rollup_config.proof_manager.prover_address,
        )
    }

    fn create_storage_manager(
        &self,
        rollup_config: &RollupConfig<<Self::Spec as Spec>::Address, Self::DaService>,
    ) -> Result<Self::StorageManager, Error> {
        NativeStorageManager::new(&rollup_config.storage.path)
    }
}

impl sov_modules_rollup_blueprint::WalletBlueprint<Native> for MockRollup<Native> {}
