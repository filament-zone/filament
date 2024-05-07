use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, CAMPAIGN_ID, CONF};

use self::exec::{
    abort_campaign, create_campaign, disperse_fees, fund_campaign, register_conversion,
    register_segment,
};
use self::query::{get_campaign, get_config, query_campaigns_by_status};

mod exec;
mod query;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:filament-zone";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn instantiate(
    deps: DepsMut,
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
    deps: DepsMut,
    _env: Env,
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
        ExecuteMsg::RegisterSegmentMsg { id, size } => register_segment(deps, info, id, size),
        ExecuteMsg::RegisterConversionsMsg { id, who } => register_conversion(deps, info, id, who),
        ExecuteMsg::DisperseFees { id } => disperse_fees(deps, info, id),
        ExecuteMsg::AbortCampaignMsg { id } => abort_campaign(deps, info, id),
    }
}

pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    // No state migrations performed, just returned a Response
    Ok(Response::default())
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
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

    use crate::{msg::GetCampaignResponse, state::{
        Auth, CampaignBudget, CampaignStatus, ConversionDesc, ConversionMechanism, ConversionProofMechanism, PayoutMechanism, SegmentDesc, SegmentKind, SegmentProofMechanism
    }};

    use super::*;

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
            .filter(|e: &&Event| e.attributes.contains(&Attribute {
                key: "campaign_id".to_string(),
                value: "1".to_string()
            }))
            .next()
            .is_some());
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
                    amount: Uint128::from(10000 as u128),
                },
                incentives: Coin {
                    denom: "tc".to_string(),
                    amount: Uint128::from(90000 as u128),
                },
            },
        };

        let bk = BankKeeper::new();
        let _ = bk.init_balance(app.storage_mut(), &admin, vec![Coin {
                denom: "tc".to_string(),
                amount: Uint128::from(100_000 as u128),
            }]);

        let _ = app.execute_contract(
            admin,
            addr.clone(),
            &msg,
            &[Coin {
                denom: "tc".to_string(),
                amount: Uint128::from(100_000 as u128),
            }],
        ).unwrap();


        let msg = QueryMsg::GetCampaign { id: 1 };
        let c: GetCampaignResponse = app.wrap().query_wasm_smart(addr.clone(), &msg).unwrap();
        assert_eq!(c.campaign.status, CampaignStatus::Indexing);
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
                    amount: Uint128::from(10000 as u128),
                },
                incentives: Coin {
                    denom: "tc".to_string(),
                    amount: Uint128::from(90000 as u128),
                },
            },
        };

        let not_admin = Addr::unchecked("not_admin");
        let bk = BankKeeper::new();
        let _ = bk.init_balance(app.storage_mut(), &not_admin, vec![Coin {
                denom: "tc".to_string(),
                amount: Uint128::from(100_000 as u128),
            }]);

        let res = app.execute_contract(
            Addr::unchecked("not_admin"),
            addr.clone(),
            &msg,
            &[Coin {
                denom: "tc".to_string(),
                amount: Uint128::from(100_000 as u128),
            }],
        ).unwrap_err();

        assert_eq!(res.downcast::<ContractError>().unwrap(), ContractError::Unauthorized {});
    }

    /* #[test]
    fn fund_campaign_insufficient_funds() {
        let (mut deps, info) = init_contract();
        let amount: u128 = 1000;
        let admin = mock_info(
            "someone",
            &[Coin {
                denom: "tc".to_string(),
                amount: Uint128::from(amount),
            }],
        );
        let campaign_id = init_campaign(deps.as_mut(), &info, &admin);

        let msg = ExecuteMsg::FundCampaignMsg {
            id: campaign_id,
            budget: CampaignBudget {
                fee: Coin {
                    denom: "tc".to_string(),
                    amount: Uint128::from(10000 as u128),
                },
                incentives: Coin {
                    denom: "tc".to_string(),
                    amount: Uint128::from(90000 as u128),
                },
            },
        };

        let res = execute(deps.as_mut(), mock_env(), admin, msg);

        assert_eq!(res, Err(ContractError::FundsBudgetMismatch {}));
    }

    #[test]
    fn register_segment() {
        let (mut deps, info) = init_contract();
        let admin = mock_info(
            "campaign_creator",
            &[Coin {
                denom: "tc".to_string(),
                amount: Uint128::from(100000 as u128),
            }],
        );
        let campaign_id = init_campaign(deps.as_mut(), &info, &admin);

        let msg = ExecuteMsg::FundCampaignMsg {
            id: campaign_id,
            budget: CampaignBudget {
                fee: Coin {
                    denom: "tc".to_string(),
                    amount: Uint128::from(10000 as u128),
                },
                incentives: Coin {
                    denom: "tc".to_string(),
                    amount: Uint128::from(90000 as u128),
                },
            },
        };

        let _ = execute(deps.as_mut(), mock_env(), admin.clone(), msg);
        assert!(INDEXING_CAMPAIGNS.has(&deps.storage, campaign_id));

        let msg = ExecuteMsg::RegisterSegmentMsg {
            id: campaign_id,
            size: 200,
        };
        let _ = execute(deps.as_mut(), mock_env(), info, msg);

        assert!(!INDEXING_CAMPAIGNS.has(&deps.storage, campaign_id));
        assert!(ATTESTING_CAMPAIGNS.has(&deps.storage, campaign_id));
    } */
}
