use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::anyhow;
use filament_hub_core::{
    campaign::{Campaign, Phase},
    criteria::{Criteria, CriteriaProposal, Criterion},
    crypto::Ed25519Signature,
    segment::{GithubSegment, SegmentData, SegmentProof},
    Budget,
    CallMessage,
    Core,
    CoreConfig,
    Event,
    Indexer,
    Segment,
};
use lazy_static::lazy_static;
use pretty_assertions::assert_eq;
use sov_bank::{get_token_id, Coins, TokenId};
use sov_modules_api::{
    prelude::UnwrapInfallible,
    test_utils::generate_address,
    Error,
    Spec,
    TxEffect,
};
use sov_test_utils::{
    generate_optimistic_runtime,
    runtime::{genesis::optimistic::HighLevelOptimisticGenesisConfig, TestRunner},
    AsUser,
    MockDaSpec,
    TestSpec,
    TestUser,
    TransactionTestCase,
};

lazy_static! {
    static ref FILA_TOKEN_ID: TokenId = {
        let salt = 0;
        let admin_addr = generate_address::<S>("admin");
        let token_name = "FILA".to_owned();
        get_token_id::<S>(&token_name, &admin_addr, salt)
    };
}

type S = TestSpec;

generate_optimistic_runtime!(TestCoreRuntime <= core: Core<S>);

struct TestRoles<S: Spec> {
    admin: TestUser<S>,
    campaign: Campaign<S>,
    campaigner: TestUser<S>,
    delegates: Vec<TestUser<S>>,
    indexer: TestUser<S>,
    relayer: TestUser<S>,
    staker: TestUser<S>,
}

#[test]
fn init_campaign() {
    let (
        TestRoles {
            campaigner,
            delegates,
            ..
        },
        mut runner,
    ) = setup();

    // Init should fail if no criteria is provided.
    {
        let campaigner = campaigner.clone();
        runner.execute_transaction(TransactionTestCase {
            input: campaigner.create_plain_message::<Core<S>>(CallMessage::Init {
                criteria: vec![],
                budget: generate_test_budget(),
                payment: None,
                evictions: vec![],
            }),
            assert: Box::new(move |result, _state| {
                assert_eq!(
                    result.tx_receipt,
                    TxEffect::Reverted(Error::ModuleError(anyhow!("missing criteria",)))
                );
            }),
        });
    }

    runner.execute_transaction(TransactionTestCase {
        input: campaigner.create_plain_message::<Core<S>>(CallMessage::Init {
            criteria: generate_test_criteria(),
            budget: generate_test_budget(),
            payment: None,
            evictions: vec![],
        }),
        assert: Box::new(move |result, state| {
            assert_eq!(result.tx_receipt, TxEffect::Successful(()));
            assert_eq!(result.events.len(), 1);
            assert_eq!(
                result.events[0],
                TestCoreRuntimeEvent::Core(Event::CampaignInitialized {
                    campaign_id: 1,
                    campaigner: campaigner.address(),
                    payment: None,
                    evictions: vec![]
                })
            );

            let expected = {
                let delegates = delegates.iter().map(|u| u.address()).collect::<Vec<_>>();
                let mut campaign = generate_test_campaign(campaigner.address());
                campaign.proposed_delegates = delegates.clone();
                campaign.delegates = delegates;
                campaign
            };
            assert_eq!(
                Core::<S>::default()
                    .get_campaign(1, state)
                    .unwrap_infallible(),
                Some(expected)
            );
        }),
    });
}

#[test]
fn propose_criteria() {
    let (
        TestRoles {
            campaigner,
            delegates,
            ..
        },
        mut runner,
    ) = setup();

    // Propose should fail if the proposer not a delegate of the campaign.
    {
        let campaigner = campaigner.clone();
        runner.execute_transaction(TransactionTestCase {
            input: campaigner.clone().create_plain_message::<Core<S>>(
                CallMessage::ProposeCriteria {
                    campaign_id: 0,
                    criteria: generate_test_criteria(),
                },
            ),
            assert: Box::new(move |result, _state| {
                assert_eq!(
                    result.tx_receipt,
                    TxEffect::Reverted(Error::ModuleError(anyhow!(
                        "invalid proposer, '{}' is not a campaign delegate",
                        campaigner.address()
                    )))
                );
            }),
        });
    }

    runner.execute_transaction(TransactionTestCase {
        input: delegates[0].create_plain_message::<Core<S>>(CallMessage::ProposeCriteria {
            campaign_id: 0,
            criteria: generate_test_criteria(),
        }),
        assert: Box::new(move |result, state| {
            assert_eq!(result.tx_receipt, TxEffect::Successful(()));
            assert_eq!(result.events.len(), 1);
            assert_eq!(
                result.events[0],
                TestCoreRuntimeEvent::Core(Event::CriteriaProposed {
                    campaign_id: 0,
                    proposer: delegates[0].address(),
                    proposal_id: 0
                })
            );

            assert_eq!(
                Core::<S>::default()
                    .get_criteria_proposal(0, 0, state)
                    .unwrap_infallible(),
                Some(CriteriaProposal {
                    campaign_id: 0,
                    proposer: delegates[0].address(),
                    criteria: generate_test_criteria(),
                })
            );
        }),
    });
}

#[test]
fn confirm_criteria() {
    let (
        TestRoles {
            campaign,
            campaigner,
            staker,
            ..
        },
        mut runner,
    ) = setup();

    // Confirm should fail if sender is not campaigner.
    {
        runner.execute_transaction(TransactionTestCase {
            input: staker.create_plain_message::<Core<S>>(CallMessage::ConfirmCriteria {
                campaign_id: 0,
                proposal_id: None,
            }),
            assert: Box::new(move |result, _state| {
                assert_eq!(
                    result.tx_receipt,
                    TxEffect::Reverted(Error::ModuleError(anyhow!(
                        "sender '{}' is not the campaigner",
                        staker.address()
                    )))
                );
            }),
        });
    }

    runner.execute_transaction(TransactionTestCase {
        input: campaigner.create_plain_message::<Core<S>>(CallMessage::ConfirmCriteria {
            campaign_id: 0,
            proposal_id: None,
        }),
        assert: Box::new(move |result, state| {
            assert_eq!(result.tx_receipt, TxEffect::Successful(()));
            assert_eq!(result.events.len(), 1);
            assert_eq!(
                result.events[0],
                TestCoreRuntimeEvent::Core(Event::CriteriaConfirmed {
                    campaign_id: 0,
                    proposal_id: None,
                })
            );

            let campaign = {
                let mut campaign = campaign.clone();
                campaign.phase = Phase::Publish;
                campaign
            };
            assert_eq!(
                Core::<S>::default()
                    .get_campaign(0, state)
                    .unwrap_infallible(),
                Some(campaign)
            );
        }),
    });
}

#[test]
fn post_segment() {
    let (
        TestRoles {
            campaigner,
            indexer,
            staker,
            campaign,
            ..
        },
        mut runner,
    ) = setup();

    // Transition to Publish phase.
    runner.execute_transaction(TransactionTestCase {
        input: campaigner.create_plain_message::<Core<S>>(CallMessage::ConfirmCriteria {
            campaign_id: 0,
            proposal_id: None,
        }),
        assert: Box::new(move |_, _| {}),
    });
    // Start indexing.
    runner.execute_transaction(TransactionTestCase {
        input: indexer
            .create_plain_message::<Core<S>>(CallMessage::IndexCampaign { campaign_id: 0 }),
        assert: Box::new(move |_, _| {}),
    });

    // Confirm that only the associated indexer can start indexing for the campaign.
    {
        runner.execute_transaction(TransactionTestCase {
            input: staker.create_plain_message::<Core<S>>(CallMessage::PostSegment {
                campaign_id: 0,
                segment: generate_test_segment(),
            }),
            assert: Box::new(move |result, _state| {
                assert_eq!(
                    result.tx_receipt,
                    TxEffect::Reverted(Error::ModuleError(anyhow!(
                        "sender '{}' is not the registered indexer '{:?}' for campaign '0'",
                        staker.address(),
                        campaign.indexer,
                    )))
                );
            }),
        });
    }

    let segment = generate_test_segment();
    runner.execute_transaction(TransactionTestCase {
        input: indexer.create_plain_message::<Core<S>>(CallMessage::PostSegment {
            campaign_id: 0,
            segment: segment.clone(),
        }),
        assert: Box::new(move |result, state| {
            assert_eq!(result.tx_receipt, TxEffect::Successful(()));
            assert_eq!(result.events.len(), 1);
            assert_eq!(
                result.events[0],
                TestCoreRuntimeEvent::Core(Event::SegmentPosted {
                    campaign_id: 0,
                    indexer: indexer.address()
                })
            );

            assert_eq!(
                Core::<S>::default()
                    .get_segment(0, state)
                    .unwrap_infallible(),
                Some(segment)
            );
        }),
    });
}

#[test]
fn indexer_registration() {
    let (TestRoles { admin, indexer, .. }, mut runner) = setup();

    {
        let indexer = indexer.clone();
        runner.execute_transaction(TransactionTestCase {
            input: admin.create_plain_message::<Core<S>>(CallMessage::RegisterIndexer(
                indexer.address(),
                "numia".to_string(),
            )),
            assert: Box::new(move |result, state| {
                assert_eq!(result.tx_receipt, TxEffect::Successful(()));
                assert_eq!(result.events.len(), 1);
                assert_eq!(
                    result.events[0],
                    TestCoreRuntimeEvent::Core(Event::IndexerRegistered {
                        addr: indexer.address(),
                        alias: "numia".to_string()
                    })
                );

                assert_eq!(
                    Core::<S>::default()
                        .get_indexer(indexer.address(), state)
                        .unwrap_infallible(),
                    Some(Indexer {
                        addr: indexer.address(),
                        alias: "numia".to_string()
                    })
                );
            }),
        });
    }

    runner.execute_transaction(TransactionTestCase {
        input: admin
            .create_plain_message::<Core<S>>(CallMessage::UnregisterIndexer(indexer.address())),
        assert: Box::new(move |result, state| {
            assert_eq!(result.tx_receipt, TxEffect::Successful(()));
            assert_eq!(result.events.len(), 1);
            assert_eq!(
                result.events[0],
                TestCoreRuntimeEvent::Core(Event::IndexerUnregistered {
                    addr: indexer.address()
                })
            );

            assert_eq!(
                Core::<S>::default()
                    .get_indexer(indexer.address(), state)
                    .unwrap_infallible(),
                None
            );
        }),
    });
}

#[test]
fn register_relayer() {
    let (TestRoles { admin, staker, .. }, mut runner) = setup();
    let relayer = generate_address::<S>("another-relayer");

    // Confirm that only the module admin can unregister a relayer..
    {
        runner.execute_transaction(TransactionTestCase {
            input: staker.create_plain_message::<Core<S>>(CallMessage::RegisterRelayer(relayer)),
            assert: Box::new(move |result, _state| {
                assert_eq!(
                    result.tx_receipt,
                    TxEffect::Reverted(Error::ModuleError(anyhow!(
                        "sender '{}' is not an admin",
                        staker.address(),
                    )))
                );
            }),
        });
    }

    runner.execute_transaction(TransactionTestCase {
        input: admin.create_plain_message::<Core<S>>(CallMessage::RegisterRelayer(relayer)),
        assert: Box::new(move |result, state| {
            assert_eq!(result.tx_receipt, TxEffect::Successful(()));
            assert_eq!(result.events.len(), 1);
            assert_eq!(
                result.events[0],
                TestCoreRuntimeEvent::Core(Event::RelayerRegistered { addr: relayer })
            );

            assert_eq!(
                Core::<S>::default()
                    .get_relayer(relayer, state)
                    .unwrap_infallible(),
                Some(relayer),
            );
        }),
    });
}

#[test]
fn unregister_relayer() {
    let (
        TestRoles {
            admin,
            relayer,
            staker,
            ..
        },
        mut runner,
    ) = setup();

    // Confirm that only the module admin can unregister a relayer..
    {
        runner.execute_transaction(TransactionTestCase {
            input: staker
                .create_plain_message::<Core<S>>(CallMessage::UnregisterIndexer(relayer.address())),
            assert: Box::new(move |result, _state| {
                assert_eq!(
                    result.tx_receipt,
                    TxEffect::Reverted(Error::ModuleError(anyhow!(
                        "sender '{}' is not an admin",
                        staker.address(),
                    )))
                );
            }),
        });
    }

    runner.execute_transaction(TransactionTestCase {
        input: admin
            .create_plain_message::<Core<S>>(CallMessage::UnregisterRelayer(relayer.address())),
        assert: Box::new(move |result, state| {
            assert_eq!(result.tx_receipt, TxEffect::Successful(()));
            assert_eq!(result.events.len(), 1);
            assert_eq!(
                result.events[0],
                TestCoreRuntimeEvent::Core(Event::RelayerUnregistered {
                    addr: relayer.address()
                })
            );

            assert_eq!(
                Core::<S>::default()
                    .get_relayer(relayer.address(), state)
                    .unwrap_infallible(),
                None
            );
        }),
    });
}

fn setup() -> (TestRoles<S>, TestRunner<TestCoreRuntime<S, MockDaSpec>, S>) {
    let genesis_config =
        HighLevelOptimisticGenesisConfig::generate().add_accounts_with_default_balance(8);

    let admin = genesis_config.additional_accounts.first().unwrap().clone();
    let staker = genesis_config.additional_accounts[1].clone();
    let indexer = genesis_config.additional_accounts[2].clone();
    let relayer = genesis_config.additional_accounts[3].clone();
    let campaigner = genesis_config.additional_accounts[4].clone();
    let delegates = vec![
        genesis_config.additional_accounts[5].clone(),
        genesis_config.additional_accounts[6].clone(),
        genesis_config.additional_accounts[7].clone(),
    ];
    let delegate_addrs = delegates.iter().map(|u| u.address()).collect::<Vec<_>>();

    let campaign = {
        let mut campaign = generate_test_campaign(campaigner.address());
        campaign.delegates = delegate_addrs.clone();
        campaign.indexer = Some(indexer.address());
        campaign
    };

    let genesis = GenesisConfig::from_minimal_config(
        genesis_config.clone().into(),
        CoreConfig {
            admin: admin.address(),
            campaigns: vec![campaign.clone()],
            delegates: delegate_addrs,
            indexers: vec![],
            relayers: vec![relayer.address()],
        },
    );

    (
        TestRoles {
            admin,
            campaign,
            campaigner,
            delegates,
            indexer,
            relayer,
            staker,
        },
        TestRunner::new_with_genesis(genesis.into_genesis_params(), TestCoreRuntime::default()),
    )
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

fn generate_test_campaign(campaigner: <S as Spec>::Address) -> Campaign<S> {
    Campaign {
        campaigner,
        phase: Phase::Criteria,

        criteria: generate_test_criteria(),
        budget: generate_test_budget(),
        payments: vec![],

        proposed_delegates: vec![],

        evictions: vec![],
        delegates: vec![],

        indexer: None,
    }
}

fn generate_test_criteria() -> Criteria {
    vec![Criterion {
        dataset_id: "osmosis".to_string(),
        parameters: Default::default(),
    }]
}

fn generate_test_segment() -> Segment {
    Segment {
        data: SegmentData::GithubSegment(GithubSegment { entries: vec![] }),
        proof: SegmentProof::Ed25519Signature(Ed25519Signature { pk: [0; 32] }),
        retrieved_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis(),
    }
}
