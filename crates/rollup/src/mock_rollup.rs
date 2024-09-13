use async_trait::async_trait;
use filament_hub_stf::runtime::{EthereumToRollupAddressConverter, Runtime};
use sov_db::{ledger_db::LedgerDb, storage_manager::NativeStorageManager};
use sov_kernels::basic::BasicKernel;
use sov_mock_da::{storable::service::StorableMockDaService, MockDaSpec};
use sov_mock_zkvm::{MockCodeCommitment, MockZkVerifier, MockZkvm};
use sov_modules_api::{
    default_spec::DefaultSpec,
    execution_mode::{ExecutionMode, Native, Zk},
    higher_kinded_types::Generic,
    runtime::capabilities::Kernel,
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
    node::da::{DaService, DaServiceWithRetries},
    zk::aggregated_proof::CodeCommitment,
};
use sov_sequencer::{FairBatchBuilderConfig, SequencerDb};
use sov_state::{DefaultStorageSpec, ProverStorage, Storage, ZkStorage};
use sov_stf_runner::{ParallelProverService, ProverService, RollupConfig, RollupProverConfig};
use tokio::sync::watch;

/// Rollup with MockDa
#[derive(Default)]
pub struct MockDemoRollup<M> {
    phantom: std::marker::PhantomData<M>,
}

impl<M: ExecutionMode> RollupBlueprint<M> for MockDemoRollup<M>
where
    DefaultSpec<Risc0Verifier, MockZkVerifier, M>: PluggableSpec,
    EthereumToRollupAddressConverter:
        TryInto<<DefaultSpec<Risc0Verifier, MockZkVerifier, M> as Spec>::Address>,
{
    type DaSpec = MockDaSpec;
    type Kernel = BasicKernel<Self::Spec, Self::DaSpec>;
    type Runtime = Runtime<Self::Spec, Self::DaSpec>;
    type Spec = DefaultSpec<Risc0Verifier, MockZkVerifier, M>;
}

#[async_trait]
impl FullNodeBlueprint<Native> for MockDemoRollup<Native> {
    type DaService = DaServiceWithRetries<StorableMockDaService>;
    type InnerZkvmHost = Risc0Host<'static>;
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
            <Self::Spec as Generic>::With<Zk>,
            Self::DaSpec,
            <MockDemoRollup<Zk> as RollupBlueprint<Zk>>::Runtime,
            <MockDemoRollup<Zk> as RollupBlueprint<Zk>>::Kernel,
        >,
    >;
    type StorageManager = NativeStorageManager<
        MockDaSpec,
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
        let mut endpoints = sov_modules_rollup_blueprint::register_endpoints::<Self, Native>(
            storage.clone(),
            ledger_db,
            sequencer_db,
            da_service,
            &rollup_config.sequencer,
        )?;

        // TODO: Add issue for Sequencer level RPC injection:
        //   https://github.com/Sovereign-Labs/sovereign-sdk-wip/issues/366
        crate::eth::register_ethereum::<Self::Spec, Self::DaService, Self::Runtime>(
            da_service.clone(),
            storage,
            &mut endpoints.jsonrpsee_module,
        )?;

        Ok(endpoints)
    }

    async fn create_da_service(
        &self,
        rollup_config: &RollupConfig<
            <Self::Spec as Spec>::Address,
            <Self::DaService as DaService>::Config,
            FairBatchBuilderConfig<Self::DaSpec>,
        >,
    ) -> Self::DaService {
        DaServiceWithRetries::new_fast(
            StorableMockDaService::from_config(rollup_config.da.clone()).await,
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
        rollup_config: &RollupConfig<
            <Self::Spec as Spec>::Address,
            <Self::DaService as DaService>::Config,
            FairBatchBuilderConfig<Self::DaSpec>,
        >,
    ) -> anyhow::Result<Self::StorageManager> {
        NativeStorageManager::new(&rollup_config.storage.path)
    }
}
