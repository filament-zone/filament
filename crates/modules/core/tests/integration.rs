use std::{
    time::{SystemTime, UNIX_EPOCH},
    vec,
};

use assert_matches::assert_matches;
use filament_hub_core::{
    campaign::{Campaign, Phase},
    criteria::{Criteria, CriteriaProposal, Criterion},
    crypto::Ed25519Signature,
    playbook::{
        Auth,
        Budget,
        ConversionDescription,
        ConversionMechanism,
        ConversionProofMechanism,
        PayoutMechanism,
        Playbook,
        SegmentDescription,
        SegmentKind,
        SegmentProofMechanism,
    },
    segment::{GithubSegment, Segment, SegmentData, SegmentProof},
    CallMessage,
    Core,
    CoreConfig,
    CoreError,
    Event,
    Indexer,
};
use lazy_static::lazy_static;
use pretty_assertions::assert_eq;
use sov_bank::{get_token_id, Coins, TokenId};
use sov_modules_api::{utils::generate_address, Context, Module, ModuleError, Spec, WorkingSet};
use sov_prover_storage_manager::new_orphan_storage;
use sov_state::{DefaultStorageSpec, ProverStorage};
use sov_test_utils::{TestHasher, TestSpec};

lazy_static! {
    static ref FILA_TOKEN_ID: TokenId = {
        let salt = 0;
        let admin_addr = generate_address::<S>("admin");
        let token_name = "FILA".to_owned();
        get_token_id::<S>(&token_name, &admin_addr, salt)
    };
}

type S = TestSpec;
pub type Storage = ProverStorage<DefaultStorageSpec<TestHasher>>;

#[test]
fn init_campaign() {
    let tmpdir = tempfile::tempdir().unwrap();
    let mut working_set = WorkingSet::new(new_orphan_storage(tmpdir.path()).unwrap());

    let sequencer_addr = generate_address::<S>("sequencer");
    let campaigner_addr = generate_address::<S>("campaigner");
    let delegate_addr = generate_address::<S>("delegate");

    let core = generate_test_core(vec![delegate_addr], vec![], vec![], &mut working_set);
    assert_eq!(None, core.get_campaign(1, &mut working_set));

    // Test RPC response.
    #[cfg(feature = "native")]
    {
        let query_response = core.rpc_get_campaign(1, &mut working_set).unwrap();
        assert_eq!(None, query_response);
    }

    // Attempted initialisation with empty criteria should fail.
    {
        let call_msg = CallMessage::Init {
            criteria: Default::default(),
            budget: generate_test_budget(),
            payment: None,
            evictions: vec![],
        };
        let context = Context::<S>::new(campaigner_addr, sequencer_addr, 1);

        assert_matches!(
            core.call(call_msg, &context, &mut working_set).unwrap_err(),
            ModuleError::ModuleError(err) => {
                assert_eq!(err.downcast::<CoreError<S>>().unwrap(), <CoreError<S>>::MissingCriteria)
            }
        );
    }

    // Init should fail if the eviction is not a proposed candidate.
    {
        let call_msg = CallMessage::Init {
            criteria: generate_test_criteria(),
            budget: generate_test_budget(),
            payment: None,
            evictions: vec![generate_address::<S>("fake_delegate")],
        };
        let context = Context::<S>::new(campaigner_addr, sequencer_addr, 2);

        assert_matches!(
            core.call(call_msg, &context, &mut working_set).unwrap_err(),
            ModuleError::ModuleError(err) => {
                assert_eq!(err.downcast::<CoreError<S>>().unwrap(), CoreError::<S>::InvalidEviction)
            }
        );
    }

    // Init campaign.
    let call_msg = CallMessage::Init {
        criteria: generate_test_criteria(),
        budget: generate_test_budget(),
        payment: None,
        evictions: vec![],
    };
    let context = Context::<S>::new(campaigner_addr, sequencer_addr, 3);
    core.call(call_msg, &context, &mut working_set).unwrap();

    let typed_event = working_set.take_event(0).unwrap();
    assert_eq!(
        typed_event.downcast::<Event<S>>().unwrap(),
        Event::CampaignInitialized {
            id: 0,
            campaigner: campaigner_addr,
            payment: None,
            evictions: vec![]
        }
    );

    let mut expected = generate_test_campaign(campaigner_addr);
    expected.phase = Phase::Criteria;
    expected.proposed_delegates = vec![delegate_addr];
    expected.delegates = vec![delegate_addr];

    assert_eq!(
        Some(expected.clone()),
        core.get_campaign(0, &mut working_set)
    );

    // Test RPC response.
    #[cfg(feature = "native")]
    {
        let query_response = core.rpc_get_campaign(0, &mut working_set).unwrap();
        assert_eq!(Some(expected), query_response);
    }
}

#[test]
fn propose_criteria() {
    let tmpdir = tempfile::tempdir().unwrap();
    let mut working_set = WorkingSet::new(new_orphan_storage(tmpdir.path()).unwrap());

    let sequencer_addr = generate_address::<S>("sequencer");
    let campaigner_addr = generate_address::<S>("campaigner");
    let delegate_addr = generate_address::<S>("delegate");

    let mut campaign = generate_test_campaign(campaigner_addr);
    campaign.phase = Phase::Criteria;
    campaign.delegates = vec![delegate_addr];

    let core = generate_test_core(
        vec![delegate_addr],
        vec![campaign],
        vec![],
        &mut working_set,
    );

    // Propose should fail if the proposer not a delegate of the campaign.
    {
        let proposer_addr = generate_address::<S>("proposer");
        let call_msg = CallMessage::ProposeCriteria {
            campaign_id: 0,
            criteria: generate_test_criteria(),
        };
        let context = Context::<S>::new(proposer_addr, sequencer_addr, 2);

        assert_matches!(
            core.call(call_msg, &context, &mut working_set).unwrap_err(),
            ModuleError::ModuleError(err) => {
                assert_eq!(
                    err.downcast::<CoreError<S>>().unwrap(),
                    CoreError::<S>::InvalidProposer { reason: format!("{} is not a campaign delegate", proposer_addr)  }
                )
            }
        );
    }

    // Propose.
    let call_msg = CallMessage::ProposeCriteria {
        campaign_id: 0,
        criteria: generate_test_criteria(),
    };
    let context = Context::<S>::new(delegate_addr, sequencer_addr, 3);
    core.call(call_msg, &context, &mut working_set).unwrap();

    let typed_event = working_set.take_event(0).unwrap();
    assert_eq!(
        typed_event.downcast::<Event<S>>().unwrap(),
        Event::CriteriaProposed {
            campaign_id: 0,
            proposer: delegate_addr,
            proposal_id: 0
        }
    );

    let expected = CriteriaProposal {
        campaign_id: 0,
        proposer: delegate_addr,
        criteria: generate_test_criteria(),
    };

    assert_eq!(
        Some(expected.clone()),
        core.get_criteria_proposal(0, 0, &mut working_set)
    );

    // Test RPC response.
    #[cfg(feature = "native")]
    {
        let query_response = core
            .rpc_get_criteria_proposal(0, 0, &mut working_set)
            .unwrap();
        assert_eq!(Some(expected), query_response);
    }
}

#[test]
fn confirm_criteria() {
    let tmpdir = tempfile::tempdir().unwrap();
    let mut working_set = WorkingSet::new(new_orphan_storage(tmpdir.path()).unwrap());

    let sequencer_addr = generate_address::<S>("sequencer");
    let campaigner_addr = generate_address::<S>("campaigner");
    let delegate_addr = generate_address::<S>("delegate");

    let mut campaign = generate_test_campaign(campaigner_addr);
    campaign.phase = Phase::Criteria;
    campaign.delegates = vec![delegate_addr];

    let core = generate_test_core(
        vec![delegate_addr],
        vec![campaign.clone()],
        vec![],
        &mut working_set,
    );

    // Confirm should fail if sender is not campaigner.
    {
        let fake_campaigner = generate_address::<S>("fake_campaigner");
        let call_msg = CallMessage::ConfirmCriteria {
            campaign_id: 0,
            proposal_id: None,
        };
        let context = Context::<S>::new(fake_campaigner, sequencer_addr, 1);

        assert_matches!(
            core.call(call_msg, &context, &mut working_set).unwrap_err(),
            ModuleError::ModuleError(err) => {
                assert_eq!(
                    err.downcast::<CoreError<S>>().unwrap(),
                    CoreError::<S>::SenderNotCampaigner { sender: fake_campaigner }
                )
            }
        );
    }

    // Confirm.
    let call_msg = CallMessage::ConfirmCriteria {
        campaign_id: 0,
        proposal_id: None,
    };
    let context = Context::<S>::new(campaigner_addr, sequencer_addr, 2);
    core.call(call_msg, &context, &mut working_set).unwrap();

    let typed_event = working_set.take_event(0).unwrap();
    assert_eq!(
        typed_event.downcast::<Event<S>>().unwrap(),
        Event::CriteriaConfirmed {
            campaign_id: 0,
            proposal_id: None,
        }
    );

    let mut expected = campaign;
    expected.phase = Phase::Publish;

    assert_eq!(
        Some(expected.clone()),
        core.get_campaign(0, &mut working_set)
    );

    // Test RPC response.
    #[cfg(feature = "native")]
    {
        let query_response = core.rpc_get_campaign(0, &mut working_set).unwrap();
        assert_eq!(Some(expected), query_response);
    }
}

#[test]
fn post_segment() {
    let tmpdir = tempfile::tempdir().unwrap();
    let mut working_set = WorkingSet::new(new_orphan_storage(tmpdir.path()).unwrap());

    let indexer_addr = generate_address::<S>("indexer");
    let sequencer_addr = generate_address::<S>("sequencer");
    let campaigner_addr = generate_address::<S>("campaigner");
    let campaign = {
        let mut campaign = generate_test_campaign(campaigner_addr);
        campaign.phase = Phase::Publish;
        campaign.indexer = Some(indexer_addr);
        campaign
    };

    let indexer = Indexer {
        addr: indexer_addr,
        alias: "Numia".to_string(),
    };

    let core = generate_test_core(
        vec![],
        vec![campaign.clone()],
        vec![indexer],
        &mut working_set,
    );
    assert_eq!(Some(campaign), core.get_campaign(0, &mut working_set));

    // Test that only the associated indexer can start indexing for the campaign.
    {
        let fake_indexer = generate_address::<S>("fake_indexer");
        let call_msg = CallMessage::IndexCampaign { id: 0 };
        let context = Context::<S>::new(fake_indexer, sequencer_addr, 1);
        // TODO(xla): Test for expected error.
        assert!(core.call(call_msg, &context, &mut working_set).is_err())
    }

    // Start campaign indexing.
    {
        let call_msg = CallMessage::IndexCampaign { id: 0 };
        let context = Context::<S>::new(indexer_addr, sequencer_addr, 2);
        core.call(call_msg, &context, &mut working_set).unwrap();

        let typed_event = working_set.take_event(0).unwrap();
        assert_eq!(
            typed_event.downcast::<Event<S>>().unwrap(),
            Event::CampaignIndexing {
                id: 0,
                indexer: indexer_addr
            }
        );

        let expected = {
            let mut campaign = generate_test_campaign(campaigner_addr);
            campaign.phase = Phase::Indexing;
            campaign.indexer = Some(indexer_addr);
            campaign
        };
        assert_eq!(Some(expected), core.get_campaign(0, &mut working_set));
    }

    // Post Segment.
    let segment = Segment {
        data: SegmentData::GithubSegment(GithubSegment { entries: vec![] }),
        proof: SegmentProof::Ed25519Signature(Ed25519Signature { pk: [0; 32] }),
        retrieved_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis(),
    };
    let call_msg = CallMessage::PostSegment {
        id: 0,
        segment: segment.clone(),
    };
    let context = Context::<S>::new(indexer_addr, sequencer_addr, 3);
    core.call(call_msg, &context, &mut working_set).unwrap();

    let typed_event = working_set.take_event(0).unwrap();
    assert_eq!(
        typed_event.downcast::<Event<S>>().unwrap(),
        Event::SegmentPosted {
            id: 0,
            indexer: indexer_addr
        }
    );

    assert_eq!(segment, core.get_segment(0, &mut working_set).unwrap());

    // Test RPC response.
    #[cfg(feature = "native")]
    {
        let query_response = core.rpc_get_segment(0, &mut working_set).unwrap();
        assert_eq!(Some(segment), query_response);
    }
}

#[test]
fn register_indexer() {
    let tmpdir = tempfile::tempdir().unwrap();
    let mut working_set = WorkingSet::new(new_orphan_storage(tmpdir.path()).unwrap());

    let admin_addr = generate_address::<S>("admin");
    let sequencer_addr = generate_address::<S>("sequencer");

    let indexer_addr = generate_address::<S>("indexer");
    let indexer_alias = "Numia".to_string();

    let core = generate_test_core(vec![], vec![], vec![], &mut working_set);

    assert_eq!(None, core.get_indexer(indexer_addr, &mut working_set));
    assert_eq!(
        Vec::<Indexer<S>>::new(),
        core.get_indexers(&mut working_set)
    );

    // Test RPC responses.
    #[cfg(feature = "native")]
    {
        let query_response = core
            .rpc_get_indexer(indexer_addr, &mut working_set)
            .unwrap();
        assert_eq!(None, query_response);

        let query_response = core.rpc_get_indexers(&mut working_set).unwrap();
        assert_eq!(Vec::<Indexer<S>>::new(), query_response);
    }

    // Call and check for event.
    {
        let context = Context::<S>::new(admin_addr, sequencer_addr, 1);
        let call_msg = CallMessage::RegisterIndexer(indexer_addr, indexer_alias.clone());
        core.call(call_msg, &context, &mut working_set).unwrap();
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
        core.get_indexer(indexer_addr, &mut working_set)
    );
    assert_eq!(vec![expected.clone()], core.get_indexers(&mut working_set));

    // Test RPC responses.
    #[cfg(feature = "native")]
    {
        let query_response = core
            .rpc_get_indexer(indexer_addr, &mut working_set)
            .unwrap();
        assert_eq!(Some(expected.clone()), query_response);

        let query_response = core.rpc_get_indexers(&mut working_set).unwrap();
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
    let indexer = Indexer {
        addr: indexer_addr.clone(),
        alias: indexer_alias.clone(),
    };

    let core = generate_test_core(vec![], vec![], vec![indexer], &mut working_set);

    // Query initial state.
    #[cfg(feature = "native")]
    {
        let query_response = core.rpc_get_indexers(&mut working_set).unwrap();
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
        core.call(call_msg, &context, &mut working_set).unwrap();
        let typed_event = working_set.take_event(0).unwrap();
        assert_eq!(
            typed_event.downcast::<Event<S>>().unwrap(),
            Event::IndexerUnregistered { addr: indexer_addr }
        );
    }

    // Test query
    #[cfg(feature = "native")]
    {
        let query_response = core.rpc_get_indexers(&mut working_set).unwrap();
        assert_eq!(Vec::<Indexer<S>>::new(), query_response);
    }
}

fn generate_test_budget() -> Budget {
    Budget {
        fee: Coins {
            amount: 100,
            token_id: *FILA_TOKEN_ID,
        },
        incentives: Coins {
            amount: 100,
            token_id: *FILA_TOKEN_ID,
        },
    }
}
fn generate_test_criteria() -> Criteria {
    vec![Criterion {
        dataset_id: "osmosis".to_string(),
        parameters: Default::default(),
    }]
}

fn generate_test_campaign(campaigner: <S as Spec>::Address) -> Campaign<S> {
    Campaign {
        campaigner,
        phase: Phase::Init,

        criteria: generate_test_criteria(),
        budget: generate_test_budget(),
        payments: vec![],

        proposed_delegates: vec![],

        evictions: vec![],
        delegates: vec![],

        indexer: None,
    }
}

fn generate_test_core(
    delegates: Vec<<S as Spec>::Address>,
    campaigns: Vec<Campaign<S>>,
    indexers: Vec<Indexer<S>>,
    working_set: &mut WorkingSet<S>,
) -> Core<S> {
    let admin = generate_address::<S>("admin");

    let core = Core::<S>::default();
    let config = CoreConfig {
        admin,

        delegates,
        campaigns,
        indexers,
    };
    core.genesis(&config, working_set).unwrap();

    core
}

#[allow(dead_code)]
fn generate_test_playbook(oracle_addr: <S as Spec>::Address) -> Playbook {
    let token_id = {
        let salt = 0;
        let token_name = "Token_New".to_owned();
        get_token_id::<S>(&token_name, &oracle_addr, salt)
    };
    Playbook {
        budget: Budget {
            fee: Coins {
                amount: 100,
                token_id,
            },
            incentives: Coins {
                amount: 100,
                token_id,
            },
        },
        segment_description: SegmentDescription {
            kind: SegmentKind::GithubTopNContributors(10),
            proof: SegmentProofMechanism::Ed25519Signature,
            sources: vec![],
        },
        conversion_description: ConversionDescription {
            kind: ConversionMechanism::Social(Auth::Github),
            proof: ConversionProofMechanism::Ed25519Signature,
        },
        payout: PayoutMechanism::ProportionalPerConversion,
        ends_at: 0,
    }
}
