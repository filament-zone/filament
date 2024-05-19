use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

use self::{
    exec::{
        abort_campaign,
        claim_incentives,
        create_campaign,
        finalized_campaign,
        fund_campaign,
        register_conversion,
        register_segment,
        set_status_indexing,
    },
    query::{get_campaign, get_config, query_campaigns_by_status},
};
use crate::{
    error::ContractError,
    msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
    state::{Config, CAMPAIGN_ID, CONF},
};

mod exec;
mod query;

// version info for migration info
#[allow(unused)]
const CONTRACT_NAME: &str = "crates.io:filament-zone";
#[allow(unused)]
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn instantiate(
    deps: DepsMut<'_>,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let cfg = Config {
        chain_id: msg.chain,
        controller: msg.controller,
        oracle: msg.oracle,
        fee_recipient: msg.fee_recipient,
    };

    CONF.save(deps.storage, &cfg)?;
    CAMPAIGN_ID.save(deps.storage, &1)?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::new())
}

pub fn execute(
    deps: DepsMut<'_>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::CreateCampaignMsg {
            admin,
            indexer,
            attester,
            segment_desc,
            conversion_desc,
            payout_mech,
            ends_at,
        } => create_campaign(
            deps,
            admin,
            indexer,
            attester,
            segment_desc,
            conversion_desc,
            payout_mech,
            ends_at,
        ),
        ExecuteMsg::FundCampaignMsg { id, budget } => fund_campaign(deps, info, id, budget),
        ExecuteMsg::SetStatusIndexingMsg { id } => set_status_indexing(deps, info, id),
        ExecuteMsg::RegisterSegmentMsg { id, size } => register_segment(deps, info, id, size),
        ExecuteMsg::RegisterConversionsMsg { id, who, amount } => {
            register_conversion(deps, env, info, id, who, amount)
        },
        ExecuteMsg::FinalizeCampaignMsg { id } => finalized_campaign(deps, env, info, id),
        ExecuteMsg::ClaimIncentivesMsg { id } => claim_incentives(deps, info, id),
        ExecuteMsg::AbortCampaignMsg { id } => abort_campaign(deps, info, id),
    }
}

pub fn migrate(_deps: DepsMut<'_>, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    // No state migrations performed, just returned a Response
    Ok(Response::default())
}

pub fn query(deps: Deps<'_>, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    let resp = match msg {
        QueryMsg::GetConfig {} => to_json_binary(&get_config(deps)?)?,
        QueryMsg::GetCampaign { id } => to_json_binary(&get_campaign(deps, id)?)?,
        QueryMsg::QueryCampaignsByStatus {
            start,
            limit,
            status,
        } => to_json_binary(&query_campaigns_by_status(deps, start, limit, status)?)?,
    };

    Ok(resp)
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{Addr, Attribute, Coin, Event, Uint128};
    use cw_multi_test::{App, BankKeeper, ContractWrapper, Executor};

    use super::*;
    use crate::{
        msg::GetCampaignResponse,
        state::{
            Auth,
            CampaignBudget,
            CampaignStatus,
            ConversionDesc,
            ConversionMechanism,
            ConversionProofMechanism,
            PayoutMechanism,
            SegmentDesc,
            SegmentKind,
            SegmentProofMechanism,
        },
    };

    #[test]
    fn proper_init() {
        let mut app = App::default();

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let creator = Addr::unchecked("creator");

        let addr = app
            .instantiate_contract(
                code_id,
                creator.clone(),
                &InstantiateMsg {
                    chain: "test-1".to_string(),
                    controller: creator.clone(),
                    oracle: creator.clone(),
                    fee_recipient: creator.clone(),
                },
                &[],
                "self",
                None,
            )
            .unwrap();

        let res: Config = app
            .wrap()
            .query_wasm_smart(addr, &QueryMsg::GetConfig {})
            .unwrap();

        assert_eq!(
            res,
            Config {
                chain_id: "test-1".to_string(),
                controller: creator.clone(),
                oracle: creator.clone(),
                fee_recipient: creator.clone(),
            },
        );
    }

    #[test]
    fn create_campaign() {
        let mut app = App::default();

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let creator = Addr::unchecked("contract_creator");

        let addr = app
            .instantiate_contract(
                code_id,
                creator.clone(),
                &InstantiateMsg {
                    chain: "test-1".to_string(),
                    controller: creator.clone(),
                    oracle: creator.clone(),
                    fee_recipient: creator.clone(),
                },
                &[],
                "self",
                None,
            )
            .unwrap();

        let admin = Addr::unchecked("campaign_admin");

        let msg = ExecuteMsg::CreateCampaignMsg {
            admin: admin.clone(),
            indexer: Addr::unchecked("indexer".to_string()),
            attester: Addr::unchecked("attester".to_string()),
            segment_desc: SegmentDesc {
                kind: SegmentKind::GithubAllContributors,
                sources: vec!["bitcoin/bitcoin".to_string()],
                proof: SegmentProofMechanism::Ed25519Signature,
            },
            conversion_desc: ConversionDesc {
                kind: ConversionMechanism::Social(Auth::Github),
                proof: ConversionProofMechanism::Ed25519Signature,
            },
            payout_mech: PayoutMechanism::ProportionalPerConversion,
            ends_at: 0,
        };

        let res = app.execute_contract(admin, addr, &msg, &[]).unwrap();
        assert!(res
            .events
            .iter()
            .any(|e: &Event| e.attributes.contains(&Attribute {
                key: "campaign_id".to_string(),
                value: "1".to_string()
            })));
    }

    #[test]
    fn fund_campaign() {
        let mut app = App::default();

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let creator = Addr::unchecked("contract_creator");

        let addr = app
            .instantiate_contract(
                code_id,
                creator.clone(),
                &InstantiateMsg {
                    chain: "test-1".to_string(),
                    controller: creator.clone(),
                    oracle: creator.clone(),
                    fee_recipient: creator.clone(),
                },
                &[],
                "self",
                None,
            )
            .unwrap();

        let admin = Addr::unchecked("campaign_admin");

        let msg = ExecuteMsg::CreateCampaignMsg {
            admin: admin.clone(),
            indexer: Addr::unchecked("indexer".to_string()),
            attester: Addr::unchecked("attester".to_string()),
            segment_desc: SegmentDesc {
                kind: SegmentKind::GithubAllContributors,
                sources: vec!["bitcoin/bitcoin".to_string()],
                proof: SegmentProofMechanism::Ed25519Signature,
            },
            conversion_desc: ConversionDesc {
                kind: ConversionMechanism::Social(Auth::Github),
                proof: ConversionProofMechanism::Ed25519Signature,
            },
            payout_mech: PayoutMechanism::ProportionalPerConversion,
            ends_at: 0,
        };

        let _ = app
            .execute_contract(admin.clone(), addr.clone(), &msg, &[])
            .unwrap();

        let msg = ExecuteMsg::FundCampaignMsg {
            id: 1,
            budget: CampaignBudget {
                fee: Coin {
                    denom: "tc".to_string(),
                    amount: Uint128::from(10000_u128),
                },
                incentives: Coin {
                    denom: "tc".to_string(),
                    amount: Uint128::from(90000_u128),
                },
            },
        };

        let bk = BankKeeper::new();
        let _ = bk.init_balance(
            app.storage_mut(),
            &admin,
            vec![Coin {
                denom: "tc".to_string(),
                amount: Uint128::from(100_000_u128),
            }],
        );

        let _ = app
            .execute_contract(
                admin,
                addr.clone(),
                &msg,
                &[Coin {
                    denom: "tc".to_string(),
                    amount: Uint128::from(100_000_u128),
                }],
            )
            .unwrap();

        let msg = QueryMsg::GetCampaign { id: 1 };
        let c: GetCampaignResponse = app.wrap().query_wasm_smart(addr.clone(), &msg).unwrap();
        assert_eq!(c.campaign.status, CampaignStatus::Funded);
    }

    #[test]
    fn fund_campaign_not_admin() {
        let mut app = App::default();

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let creator = Addr::unchecked("contract_creator");

        let addr = app
            .instantiate_contract(
                code_id,
                creator.clone(),
                &InstantiateMsg {
                    chain: "test-1".to_string(),
                    controller: creator.clone(),
                    oracle: creator.clone(),
                    fee_recipient: creator.clone(),
                },
                &[],
                "self",
                None,
            )
            .unwrap();

        let admin = Addr::unchecked("campaign_admin");

        let msg = ExecuteMsg::CreateCampaignMsg {
            admin: admin.clone(),
            indexer: Addr::unchecked("indexer".to_string()),
            attester: Addr::unchecked("attester".to_string()),
            segment_desc: SegmentDesc {
                kind: SegmentKind::GithubAllContributors,
                sources: vec!["bitcoin/bitcoin".to_string()],
                proof: SegmentProofMechanism::Ed25519Signature,
            },
            conversion_desc: ConversionDesc {
                kind: ConversionMechanism::Social(Auth::Github),
                proof: ConversionProofMechanism::Ed25519Signature,
            },
            payout_mech: PayoutMechanism::ProportionalPerConversion,
            ends_at: 0,
        };

        let _ = app
            .execute_contract(admin.clone(), addr.clone(), &msg, &[])
            .unwrap();

        let msg = ExecuteMsg::FundCampaignMsg {
            id: 1,
            budget: CampaignBudget {
                fee: Coin {
                    denom: "tc".to_string(),
                    amount: Uint128::from(10000_u128),
                },
                incentives: Coin {
                    denom: "tc".to_string(),
                    amount: Uint128::from(90000_u128),
                },
            },
        };

        let not_admin = Addr::unchecked("not_admin");
        let bk = BankKeeper::new();
        let _ = bk.init_balance(
            app.storage_mut(),
            &not_admin,
            vec![Coin {
                denom: "tc".to_string(),
                amount: Uint128::from(100_000_u128),
            }],
        );

        let res = app
            .execute_contract(
                Addr::unchecked("not_admin"),
                addr.clone(),
                &msg,
                &[Coin {
                    denom: "tc".to_string(),
                    amount: Uint128::from(100_000_u128),
                }],
            )
            .unwrap_err();

        assert_eq!(
            res.downcast::<ContractError>().unwrap(),
            ContractError::Unauthorized {}
        );
    }

    #[test]
    fn fund_campaign_insufficient_funds() {
        let mut app = App::default();

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let creator = Addr::unchecked("contract_creator");

        let addr = app
            .instantiate_contract(
                code_id,
                creator.clone(),
                &InstantiateMsg {
                    chain: "test-1".to_string(),
                    controller: creator.clone(),
                    oracle: creator.clone(),
                    fee_recipient: creator.clone(),
                },
                &[],
                "self",
                None,
            )
            .unwrap();

        let admin = Addr::unchecked("campaign_admin");

        let msg = ExecuteMsg::CreateCampaignMsg {
            admin: admin.clone(),
            indexer: Addr::unchecked("indexer".to_string()),
            attester: Addr::unchecked("attester".to_string()),
            segment_desc: SegmentDesc {
                kind: SegmentKind::GithubAllContributors,
                sources: vec!["bitcoin/bitcoin".to_string()],
                proof: SegmentProofMechanism::Ed25519Signature,
            },
            conversion_desc: ConversionDesc {
                kind: ConversionMechanism::Social(Auth::Github),
                proof: ConversionProofMechanism::Ed25519Signature,
            },
            payout_mech: PayoutMechanism::ProportionalPerConversion,
            ends_at: 0,
        };

        let _ = app
            .execute_contract(admin.clone(), addr.clone(), &msg, &[])
            .unwrap();

        let msg = ExecuteMsg::FundCampaignMsg {
            id: 1,
            budget: CampaignBudget {
                fee: Coin {
                    denom: "tc".to_string(),
                    amount: Uint128::from(10000_u128),
                },
                incentives: Coin {
                    denom: "tc".to_string(),
                    amount: Uint128::from(90000_u128),
                },
            },
        };

        let not_admin = Addr::unchecked("not_admin");
        let bk = BankKeeper::new();
        let _ = bk.init_balance(
            app.storage_mut(),
            &not_admin,
            vec![Coin {
                denom: "tc".to_string(),
                amount: Uint128::from(100_000_u128),
            }],
        );

        let res = app
            .execute_contract(
                Addr::unchecked("not_admin"),
                addr.clone(),
                &msg,
                &[Coin {
                    denom: "tc".to_string(),
                    amount: Uint128::from(80_000_u128),
                }],
            )
            .unwrap_err();

        assert_eq!(
            res.downcast::<ContractError>().unwrap(),
            ContractError::FundsBudgetMismatch {}
        );
    }

    #[test]
    fn register_segment() {
        let mut app = App::default();

        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));

        let creator = Addr::unchecked("contract_creator");

        let addr = app
            .instantiate_contract(
                code_id,
                creator.clone(),
                &InstantiateMsg {
                    chain: "test-1".to_string(),
                    controller: creator.clone(),
                    oracle: creator.clone(),
                    fee_recipient: creator.clone(),
                },
                &[],
                "self",
                None,
            )
            .unwrap();

        let admin = Addr::unchecked("campaign_admin");

        let msg = ExecuteMsg::CreateCampaignMsg {
            admin: admin.clone(),
            indexer: Addr::unchecked("indexer".to_string()),
            attester: Addr::unchecked("attester".to_string()),
            segment_desc: SegmentDesc {
                kind: SegmentKind::GithubAllContributors,
                sources: vec!["bitcoin/bitcoin".to_string()],
                proof: SegmentProofMechanism::Ed25519Signature,
            },
            conversion_desc: ConversionDesc {
                kind: ConversionMechanism::Social(Auth::Github),
                proof: ConversionProofMechanism::Ed25519Signature,
            },
            payout_mech: PayoutMechanism::ProportionalPerConversion,
            ends_at: 0,
        };

        let _ = app
            .execute_contract(admin.clone(), addr.clone(), &msg, &[])
            .unwrap();

        let msg = ExecuteMsg::FundCampaignMsg {
            id: 1,
            budget: CampaignBudget {
                fee: Coin {
                    denom: "tc".to_string(),
                    amount: Uint128::from(10000_u128),
                },
                incentives: Coin {
                    denom: "tc".to_string(),
                    amount: Uint128::from(90000_u128),
                },
            },
        };

        let bk = BankKeeper::new();
        let _ = bk.init_balance(
            app.storage_mut(),
            &admin,
            vec![Coin {
                denom: "tc".to_string(),
                amount: Uint128::from(100_000_u128),
            }],
        );

        let _ = app
            .execute_contract(
                admin.clone(),
                addr.clone(),
                &msg,
                &[Coin {
                    denom: "tc".to_string(),
                    amount: Uint128::from(100_000_u128),
                }],
            )
            .unwrap();

        let msg = ExecuteMsg::SetStatusIndexingMsg { id: 1 };
        let _ = app.execute_contract(creator.clone(), addr.clone(), &msg, &[]);

        let msg = ExecuteMsg::RegisterSegmentMsg { id: 1, size: 200 };
        let res = app.execute_contract(creator, addr.clone(), &msg, &[]);
        assert!(res.is_ok());

        let msg = QueryMsg::GetCampaign { id: 1 };
        let c: GetCampaignResponse = app.wrap().query_wasm_smart(addr.clone(), &msg).unwrap();
        assert_eq!(c.campaign.status, CampaignStatus::Attesting);
        assert_eq!(c.campaign.segment_size, 200);
    }
}
