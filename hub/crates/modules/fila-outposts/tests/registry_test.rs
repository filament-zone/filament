use sov_modules_api::default_context::DefaultContext;
use sov_modules_api::utils::generate_address as gen_addr_generic;
use sov_modules_api::{Address, Context, Module, WorkingSet};
use sov_prover_storage_manager::{new_orphan_storage, SnapshotManager};
use sov_rollup_interface::stf::Event;
use sov_state::{DefaultStorageSpec, ProverStorage};

use fila_outposts::{CallMessage, OutpostRegistry, OutpostRegistryConfig, OutpostResponse};

pub type C = DefaultContext;
pub type Storage = ProverStorage<DefaultStorageSpec, SnapshotManager>;
fn generate_address(name: &str) -> Address {
    gen_addr_generic::<DefaultContext>(name)
}

#[test]
fn genesis_and_register() {
    // Preparation
    let admin = generate_address("admin");
    let deployer = generate_address("deployer");
    let sequencer = generate_address("sequencer");
    let config: OutpostRegistryConfig<C> = OutpostRegistryConfig { admin };

    let tmpdir = tempfile::tempdir().unwrap();
    let mut working_set = WorkingSet::new(new_orphan_storage(tmpdir.path()).unwrap());
    let registry = OutpostRegistry::default();

    // Genesis
    let genesis_result = registry.genesis(&config, &mut working_set);
    assert!(genesis_result.is_ok());

    let res: Option<OutpostResponse> = registry
        .get_outpost("neutron-1".to_owned(), &mut working_set)
        .unwrap();
    assert!(res.is_none());

    // Register
    let register_message = CallMessage::Register {
        chain_id: "neutron-1".to_owned(),
    };
    let deployer_ctx = C::new(deployer, sequencer, 1);
    registry
        .call(register_message.clone(), &deployer_ctx, &mut working_set)
        .expect("registration failed");

    assert_eq!(
        working_set.events()[0],
        Event::new(
            "OutpostRegistry register",
            "An outpost with chain_id neutron-1 was registered"
        )
    );
    let res: Option<OutpostResponse> = registry
        .get_outpost("neutron-1".to_owned(), &mut working_set)
        .unwrap();
    assert_eq!(
        res,
        Some(OutpostResponse {
            chain_id: "neutron-1".to_owned()
        })
    );

    let register_attempt = registry.call(register_message, &deployer_ctx, &mut working_set);

    assert!(register_attempt.is_err());
    let error_message = register_attempt.err().unwrap().to_string();
    assert_eq!(
        "Outpost with chain_id neutron-1 already exists",
        error_message
    );
}
