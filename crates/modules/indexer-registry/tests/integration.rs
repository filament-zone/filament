use filament_hub_indexer_registry::{
    CallMessage,
    Event,
    Indexer,
    IndexerRegistry,
    IndexerRegistryConfig,
};
use sov_modules_api::{utils::generate_address, Context, Module, WorkingSet};
use sov_prover_storage_manager::new_orphan_storage;
use sov_state::{DefaultStorageSpec, ProverStorage};
use sov_test_utils::TestHasher;

type S = sov_test_utils::TestSpec;
pub type Storage = ProverStorage<DefaultStorageSpec<TestHasher>>;

#[test]
fn register_indexer() {
    let tmpdir = tempfile::tempdir().unwrap();
    let mut working_set = WorkingSet::new(new_orphan_storage(tmpdir.path()).unwrap());

    let admin_addr = generate_address::<S>("admin");
    let sequencer_addr = generate_address::<S>("sequencer");

    let indexer_addr = generate_address::<S>("indexer");
    let indexer_alias = "Numia".to_string();

    let indexer_registry = IndexerRegistry::<S>::default();
    let indexer_registry_config = IndexerRegistryConfig {
        admin: admin_addr,
        indexers: vec![],
    };
    indexer_registry
        .genesis(&indexer_registry_config, &mut working_set)
        .unwrap();

    assert_eq!(
        None,
        indexer_registry.get_indexer(indexer_addr, &mut working_set)
    );
    assert_eq!(
        Vec::<Indexer<S>>::new(),
        indexer_registry.get_indexers(&mut working_set)
    );

    // Test RPC responses.
    #[cfg(feature = "native")]
    {
        let query_response = indexer_registry
            .rpc_get_indexer(indexer_addr, &mut working_set)
            .unwrap();
        assert_eq!(None, query_response);

        let query_response = indexer_registry.rpc_get_indexers(&mut working_set).unwrap();
        assert_eq!(Vec::<Indexer<S>>::new(), query_response);
    }

    // Call and check for event.
    {
        let context = Context::<S>::new(admin_addr, sequencer_addr, 1);
        let call_msg = CallMessage::RegisterIndexer(indexer_addr, indexer_alias.clone());
        indexer_registry
            .call(call_msg, &context, &mut working_set)
            .unwrap();
        let typed_event = working_set.take_event(0).unwrap();
        assert_eq!(
            typed_event.downcast::<Event<S>>().unwrap(),
            Event::IndexerRegistered {
                addr: indexer_addr,
                alias: indexer_alias.clone(),
            }
        );
    }

    let expected = Indexer {
        addr: indexer_addr,
        alias: indexer_alias,
    };
    assert_eq!(
        Some(expected.clone()),
        indexer_registry.get_indexer(indexer_addr, &mut working_set)
    );
    assert_eq!(
        vec![expected.clone()],
        indexer_registry.get_indexers(&mut working_set)
    );

    // Test RPC responses.
    #[cfg(feature = "native")]
    {
        let query_response = indexer_registry
            .rpc_get_indexer(indexer_addr, &mut working_set)
            .unwrap();
        assert_eq!(Some(expected.clone()), query_response);

        let query_response = indexer_registry.rpc_get_indexers(&mut working_set).unwrap();
        assert_eq!(vec![expected], query_response);
    }
}

#[test]
fn unregister_indexer() {
    let tmpdir = tempfile::tempdir().unwrap();
    let mut working_set = WorkingSet::new(new_orphan_storage(tmpdir.path()).unwrap());

    let admin_addr = generate_address::<S>("admin");
    let sequencer_addr = generate_address::<S>("sequencer");

    let indexer_addr = generate_address::<S>("indexer");
    let indexer_alias = "Numia".to_string();

    let indexer_registry = IndexerRegistry::<S>::default();
    let indexer_registry_config = IndexerRegistryConfig {
        admin: admin_addr,
        indexers: vec![(indexer_addr, indexer_alias.clone())],
    };
    indexer_registry
        .genesis(&indexer_registry_config, &mut working_set)
        .unwrap();

    // Query initial state.
    #[cfg(feature = "native")]
    {
        let query_response = indexer_registry.rpc_get_indexers(&mut working_set).unwrap();
        assert_eq!(
            vec![Indexer {
                addr: indexer_addr,
                alias: indexer_alias.clone(),
            }],
            query_response
        );
    }

    {
        let context = Context::<S>::new(admin_addr, sequencer_addr, 1);
        let call_msg = CallMessage::UnregisterIndexer(indexer_addr);
        indexer_registry
            .call(call_msg, &context, &mut working_set)
            .unwrap();
        let typed_event = working_set.take_event(0).unwrap();
        assert_eq!(
            typed_event.downcast::<Event<S>>().unwrap(),
            Event::IndexerUnregistered { addr: indexer_addr }
        );
    }

    // Test query
    #[cfg(feature = "native")]
    {
        let query_response = indexer_registry.rpc_get_indexers(&mut working_set).unwrap();
        assert_eq!(Vec::<Indexer<S>>::new(), query_response);
    }
}
