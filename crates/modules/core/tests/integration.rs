use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::anyhow;
use filament_hub_core::{
    campaign::{Campaign, Phase},
    criteria::{Criteria, CriteriaProposal, Criterion, CriterionCategory},
    crypto::Ed25519Signature,
    segment::{SegmentData, SegmentProof},
    CallMessage,
    Core,
    CoreConfig,
    Event,
    Indexer,
    Segment,
};
use lazy_static::lazy_static;
use pretty_assertions::assert_eq;
use sov_bank::{get_token_id, TokenId};
use sov_modules_api::{prelude::UnwrapInfallible, Error, GasUnit, Spec, TxEffect};
use sov_modules_stf_blueprint::RevertedTxContents;
use sov_test_utils::{
    generate_optimistic_runtime,
    runtime::{genesis::optimistic::HighLevelOptimisticGenesisConfig, TestRunner},
    AsUser,
    MockDaSpec,
    TestSpec,
    TestUser,
    TransactionTestCase,
};

type S = TestSpec;

generate_optimistic_runtime!(TestCoreRuntime <= core: Core<S>);

lazy_static! {
    static ref FILA_TOKEN_ID: TokenId = {
        let salt = 0;
        let admin = TestUser::<S>::generate_with_default_balance();
        let token_name = "FILA".to_owned();
        get_token_id::<S>(&token_name, &admin.address(), salt)
    };
}

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
                title: "".to_string(),
                description: "".to_string(),
                criteria: vec![],
                evictions: vec![],
            }),
            assert: Box::new(move |result, _state| {
                assert_eq!(
                    result.tx_receipt,
                    TxEffect::Reverted(RevertedTxContents {
                        gas_used: GasUnit::from([100, 100]),
                        reason: Error::ModuleError(anyhow!("missing criteria"))
                    })
                );
            }),
        });
    }

    runner.execute_transaction(TransactionTestCase {
        input: campaigner.create_plain_message::<Core<S>>(CallMessage::Init {
            title: "".to_string(),
            description: "".to_string(),
            criteria: generate_test_criteria(),
            evictions: vec![],
        }),
        assert: Box::new(move |result, state| {
            assert!(result.tx_receipt.is_successful());
            assert_eq!(result.events.len(), 1);
            assert_eq!(
                result.events[0],
                TestCoreRuntimeEvent::Core(Event::CampaignInitialized {
                    campaign_id: 1,
                    campaigner: campaigner.address(),
                    evictions: vec![]
                })
            );

            let expected = {
                let delegates = delegates.iter().map(|u| u.address()).collect::<Vec<_>>();
                let mut campaign = generate_test_campaign(campaigner.address());
                campaign.delegates = delegates;
                campaign
            };
            assert_eq!(
                Core::<S>::default()
                    .get_campaign(1, state)
                    .unwrap_infallible(),
                Some(expected.clone())
            );
            assert_eq!(
                Core::<S>::default()
                    .get_campaigns_by_addr(campaigner.address(), state)
                    .unwrap_infallible(),
                vec![expected]
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
                    TxEffect::Reverted(RevertedTxContents {
                        gas_used: GasUnit::from([100, 100]),
                        reason: Error::ModuleError(anyhow!(
                            "invalid proposer, '{}' is not a campaign delegate",
                            campaigner.address()
                        ))
                    })
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
            assert!(result.tx_receipt.is_successful());
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
                    TxEffect::Reverted(RevertedTxContents {
                        gas_used: GasUnit::from([100, 100]),
                        reason: Error::ModuleError(anyhow!(
                            "sender '{}' is not the campaigner",
                            staker.address()
                        ))
                    })
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
            assert!(result.tx_receipt.is_successful());
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
                    TxEffect::Reverted(RevertedTxContents {
                        gas_used: GasUnit::from([100, 100]),
                        reason: Error::ModuleError(anyhow!(
                            "sender '{}' is not the registered indexer '{:?}' for campaign '0'",
                            staker.address(),
                            campaign.indexer,
                        ))
                    })
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
            assert!(result.tx_receipt.is_successful());
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
        let admin = admin.clone();
        let indexer = indexer.clone();
        runner.execute_transaction(TransactionTestCase {
            input: admin.create_plain_message::<Core<S>>(CallMessage::RegisterIndexer {
                address: indexer.address(),
                alias: "numia".to_string(),
            }),
            assert: Box::new(move |result, state| {
                assert!(result.tx_receipt.is_successful());
                assert_eq!(result.events.len(), 1);
                assert_eq!(
                    result.events[0],
                    TestCoreRuntimeEvent::Core(Event::IndexerRegistered {
                        addr: indexer.address(),
                        alias: "numia".to_string(),
                        sender: admin.address(),
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
        input: admin.create_plain_message::<Core<S>>(CallMessage::UnregisterIndexer {
            address: indexer.address(),
        }),
        assert: Box::new(move |result, state| {
            assert!(result.tx_receipt.is_successful());
            assert_eq!(result.events.len(), 1);
            assert_eq!(
                result.events[0],
                TestCoreRuntimeEvent::Core(Event::IndexerUnregistered {
                    addr: indexer.address(),
                    sender: admin.address(),
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
    let relayer = TestUser::<S>::generate_with_default_balance();

    // Confirm that only the module admin can unregister a relayer..
    {
        runner.execute_transaction(TransactionTestCase {
            input: staker.create_plain_message::<Core<S>>(CallMessage::RegisterRelayer {
                address: relayer.address(),
            }),
            assert: Box::new(move |result, _state| {
                assert_eq!(
                    result.tx_receipt,
                    TxEffect::Reverted(RevertedTxContents {
                        gas_used: GasUnit::from([100, 100]),
                        reason: Error::ModuleError(anyhow!(
                            "sender '{}' is not an admin",
                            staker.address(),
                        ))
                    })
                );
            }),
        });
    }

    runner.execute_transaction(TransactionTestCase {
        input: admin.create_plain_message::<Core<S>>(CallMessage::RegisterRelayer {
            address: relayer.address(),
        }),
        assert: Box::new(move |result, state| {
            assert!(result.tx_receipt.is_successful());
            assert_eq!(result.events.len(), 1);
            assert_eq!(
                result.events[0],
                TestCoreRuntimeEvent::Core(Event::RelayerRegistered {
                    addr: relayer.address(),
                    sender: admin.address()
                })
            );

            assert_eq!(
                Core::<S>::default()
                    .get_relayer(relayer.address(), state)
                    .unwrap_infallible(),
                Some(relayer.address()),
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

    // Confirm that only the module admin can unregister a relayer.
    {
        runner.execute_transaction(TransactionTestCase {
            input: staker.create_plain_message::<Core<S>>(CallMessage::UnregisterIndexer {
                address: relayer.address(),
            }),
            assert: Box::new(move |result, _state| {
                assert_eq!(
                    result.tx_receipt,
                    TxEffect::Reverted(RevertedTxContents {
                        gas_used: GasUnit::from([100, 100]),
                        reason: Error::ModuleError(anyhow!(
                            "sender '{}' is not an admin",
                            staker.address(),
                        ))
                    })
                );
            }),
        });
    }

    runner.execute_transaction(TransactionTestCase {
        input: admin.create_plain_message::<Core<S>>(CallMessage::UnregisterRelayer {
            address: relayer.address(),
        }),
        assert: Box::new(move |result, state| {
            assert!(result.tx_receipt.is_successful());
            assert_eq!(result.events.len(), 1);
            assert_eq!(
                result.events[0],
                TestCoreRuntimeEvent::Core(Event::RelayerUnregistered {
                    addr: relayer.address(),
                    sender: admin.address(),
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

#[test]
fn update_voting_power() {
    let (
        TestRoles {
            delegates,
            relayer,
            staker,
            ..
        },
        mut runner,
    ) = setup();

    // Confirm that only a registered relayer can update voting powers.
    {
        runner.execute_transaction(TransactionTestCase {
            input: staker.create_plain_message::<Core<S>>(CallMessage::UpdateVotingPower {
                address: delegates[0].address(),
                power: 1000,
            }),
            assert: Box::new(move |result, _state| {
                assert_eq!(
                    result.tx_receipt,
                    TxEffect::Reverted(RevertedTxContents {
                        gas_used: GasUnit::from([100, 100]),
                        reason: Error::ModuleError(anyhow!(
                            "sender '{}' is not a registered relayer",
                            staker.address(),
                        )),
                    })
                );
            }),
        });
    }

    {
        let delegate = delegates[0].address();
        let relayer = relayer.clone();

        runner.execute_transaction(TransactionTestCase {
            input: relayer.create_plain_message::<Core<S>>(CallMessage::UpdateVotingPower {
                address: delegate,
                power: 1000,
            }),
            assert: Box::new(move |result, state| {
                assert!(result.tx_receipt.is_successful());
                assert_eq!(result.events.len(), 1);
                assert_eq!(
                    result.events[0],
                    TestCoreRuntimeEvent::Core(Event::VotingPowerUpdated {
                        addr: delegate,
                        power: 1000,
                        relayer: relayer.address()
                    })
                );

                assert_eq!(
                    Core::<S>::default()
                        .get_voting_power(delegate, state)
                        .unwrap_infallible(),
                    1000
                );
            }),
        });
    }

    // Update voting power of remaining delegates.
    {
        let delegate = delegates[1].address();
        let relayer = relayer.clone();

        runner.execute_transaction(TransactionTestCase {
            input: relayer.create_plain_message::<Core<S>>(CallMessage::UpdateVotingPower {
                address: delegate,
                power: 10000,
            }),
            assert: Box::new(move |result, _| {
                assert!(result.tx_receipt.is_successful());
            }),
        });
    }

    // Ensure voting power is ordered from highest to lowest.
    {
        let delegate = delegates[2].address();

        runner.execute_transaction(TransactionTestCase {
            input: relayer.create_plain_message::<Core<S>>(CallMessage::UpdateVotingPower {
                address: delegate,
                power: 8000,
            }),
            assert: Box::new(move |result, state| {
                assert!(result.tx_receipt.is_successful());

                assert_eq!(
                    Core::<S>::default()
                        .get_voting_powers(state)
                        .unwrap_infallible(),
                    vec![
                        (delegates[1].address(), 10000),
                        (delegates[2].address(), 8000),
                        (delegates[0].address(), 1000),
                    ],
                );
            }),
        });
    }
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
        campaign.delegates.clone_from(&delegate_addrs);
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
            powers: Default::default(),
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

fn generate_test_campaign(campaigner: <S as Spec>::Address) -> Campaign<S> {
    Campaign {
        campaigner,
        phase: Phase::Criteria,

        title: "".to_string(),
        description: "".to_string(),

        criteria: generate_test_criteria(),

        evictions: vec![],
        delegates: vec![],

        indexer: None,
    }
}

fn generate_test_criteria() -> Criteria {
    vec![Criterion {
        name: "Test Criterion".to_string(),
        category: CriterionCategory::Balance,
        parameters: Default::default(),
        weight: 1,
    }]
}

fn generate_test_segment() -> Segment {
    Segment {
        data: SegmentData::Plain {
            allocations: vec![],
        },
        proof: SegmentProof::Ed25519Signature(Ed25519Signature { pk: [0; 32] }),
        retrieved_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis(),
    }
}
