use std::time::{SystemTime, UNIX_EPOCH};

use filament_hub_campaigns::{
    campaign::{Campaign, Status},
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
    Campaigns,
    CampaignsConfig,
    Event,
};
use filament_hub_indexer_registry::IndexerRegistryConfig;
use sov_bank::{get_token_id, Coins};
use sov_modules_api::{utils::generate_address, Context, Module, Spec, WorkingSet};
use sov_prover_storage_manager::new_orphan_storage;
use sov_state::{DefaultStorageSpec, ProverStorage};
use sov_test_utils::{TestHasher, TestSpec};

type S = TestSpec;
pub type Storage = ProverStorage<DefaultStorageSpec<TestHasher>>;

#[test]
fn create_campaign() {
    let tmpdir = tempfile::tempdir().unwrap();
    let mut working_set = WorkingSet::new(new_orphan_storage(tmpdir.path()).unwrap());

    let attester_addr = generate_address::<S>("attester");
    let indexer_addr = generate_address::<S>("indexer");
    let sequencer_addr = generate_address::<S>("sequencer");
    let oracle_addr = generate_address::<S>("oracle");
    let origin = "neutron".to_string();
    let origin_id = 1;
    let playbook = generate_test_playbook(oracle_addr);

    let campaigns = generate_test_campaigns(indexer_addr, vec![], &mut working_set);
    assert_eq!(None, campaigns.get_campaign(1, &mut working_set));

    // Test RPC response.
    #[cfg(feature = "native")]
    {
        let query_response = campaigns.rpc_get_campaign(1, &mut working_set).unwrap();
        assert_eq!(None, query_response);
    }

    // Attempt creation with missing indexer.
    {
        let call_msg = CallMessage::CreateCampaign {
            origin: origin.clone(),
            origin_id,
            indexer: generate_address::<S>("unregistered_indexer"),
            attester: attester_addr,
            playbook: playbook.clone(),
        };

        let context = Context::<S>::new(oracle_addr, sequencer_addr, 1);
        assert_eq!(
            true,
            campaigns
                .call(call_msg, &context, &mut working_set)
                .is_err()
        );
    }

    // Create campaign.
    let call_msg = CallMessage::CreateCampaign {
        origin,
        origin_id,
        indexer: indexer_addr,
        attester: attester_addr,
        playbook: playbook.clone(),
    };
    let context = Context::<S>::new(oracle_addr, sequencer_addr, 2);
    campaigns
        .call(call_msg, &context, &mut working_set)
        .unwrap();

    let typed_event = working_set.take_event(0).unwrap();
    assert_eq!(
        typed_event.downcast::<Event<S>>().unwrap(),
        Event::CampaignCreated {
            id: 1,
            origin: "neutron".to_string(),
            origin_id: 1
        }
    );

    let expected = Campaign {
        status: Status::Funded,
        origin: "neutron".to_string(),
        origin_id: 1,
        indexer: indexer_addr,
        attester: attester_addr,
        playbook,
    };

    assert_eq!(
        Some(expected.clone()),
        campaigns.get_campaign(1, &mut working_set)
    );

    // Test RPC response.
    #[cfg(feature = "native")]
    {
        let query_response = campaigns.rpc_get_campaign(1, &mut working_set).unwrap();
        assert_eq!(Some(expected), query_response);
    }
}

#[test]
fn post_segment() {
    let tmpdir = tempfile::tempdir().unwrap();
    let mut working_set = WorkingSet::new(new_orphan_storage(tmpdir.path()).unwrap());

    let attester_addr = generate_address::<S>("attester");
    let indexer_addr = generate_address::<S>("indexer");
    let oracle_addr = generate_address::<S>("oracle");
    let sequencer_addr = generate_address::<S>("sequencer");
    let playbook = generate_test_playbook(oracle_addr);
    let campaign = Campaign {
        status: Status::Funded,
        origin: "neutron".to_string(),
        origin_id: 1,
        indexer: indexer_addr,
        attester: attester_addr,
        playbook: playbook.clone(),
    };

    let campaigns = generate_test_campaigns(indexer_addr, vec![campaign.clone()], &mut working_set);
    assert_eq!(Some(campaign), campaigns.get_campaign(1, &mut working_set));

    // Test that only the associated indexer can start indexing for the campaign.
    {
        let fake_indexer = generate_address::<S>("fake_indexer");
        let call_msg = CallMessage::IndexCampaign { id: 1 };
        let context = Context::<S>::new(fake_indexer, sequencer_addr, 1);
        // TODO(xla): Test for expected error.
        assert!(campaigns
            .call(call_msg, &context, &mut working_set)
            .is_err())
    }

    // Start campaign indexing.
    {
        let call_msg = CallMessage::IndexCampaign { id: 1 };
        let context = Context::<S>::new(indexer_addr, sequencer_addr, 2);
        campaigns
            .call(call_msg, &context, &mut working_set)
            .unwrap();

        let typed_event = working_set.take_event(0).unwrap();
        assert_eq!(
            typed_event.downcast::<Event<S>>().unwrap(),
            Event::CampaignIndexing {
                id: 1,
                indexer: indexer_addr
            }
        );

        let expected = Campaign {
            status: Status::Indexing,
            origin: "neutron".to_string(),
            origin_id: 1,
            indexer: indexer_addr,
            attester: attester_addr,
            playbook,
        };
        assert_eq!(Some(expected), campaigns.get_campaign(1, &mut working_set));
    }

    // Post Segment.
    let segment = Segment {
        data: SegmentData::GithubSegment(GithubSegment { entries: vec![] }),
        proof: SegmentProof::Ed25519Signature(Ed25519Signature {
            pk: [0; 32],
            sig: [0; 64],
        }),
        retrieved_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis(),
    };
    let call_msg = CallMessage::PostSegment {
        id: 1,
        segment: segment.clone(),
    };
    let context = Context::<S>::new(indexer_addr, sequencer_addr, 3);
    campaigns
        .call(call_msg, &context, &mut working_set)
        .unwrap();

    let typed_event = working_set.take_event(0).unwrap();
    assert_eq!(
        typed_event.downcast::<Event<S>>().unwrap(),
        Event::SegmentPosted {
            id: 1,
            indexer: indexer_addr
        }
    );

    assert_eq!(segment, campaigns.get_segment(1, &mut working_set).unwrap());

    // Test RPC response.
    #[cfg(feature = "native")]
    {
        let query_response = campaigns.rpc_get_segment(1, &mut working_set).unwrap();
        assert_eq!(Some(segment), query_response);
    }
}

fn generate_test_campaigns(
    indexer_addr: <S as Spec>::Address,
    initial_campaigns: Vec<Campaign<S>>,
    working_set: &mut WorkingSet<S>,
) -> Campaigns<S> {
    let admin_addr = generate_address::<S>("admin");
    let indexer_alias = "Numia".to_string();

    let campaigns = Campaigns::<S>::default();
    let campaigns_config = CampaignsConfig {
        campaigns: initial_campaigns,
    };
    campaigns.genesis(&campaigns_config, working_set).unwrap();

    campaigns
        .indexer_registry
        .genesis(
            &IndexerRegistryConfig {
                admin: admin_addr,
                indexers: vec![(indexer_addr, indexer_alias)],
            },
            working_set,
        )
        .unwrap();

    campaigns
}

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
