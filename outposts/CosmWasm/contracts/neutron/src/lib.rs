#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response};
use msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

pub mod contract;
mod error;
pub mod msg;
pub mod state;

pub use crate::error::ContractError;

// Version info for migrations.
const CONTRACT_NAME: &str = "crates.io:filament-incentives-registry";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type Result<T> = core::result::Result<T, ContractError>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut<'_>,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response> {
    contract::execute(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut<'_>,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response> {
    contract::instantiate(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut<'_>, env: Env, msg: MigrateMsg) -> Result<Response> {
    contract::migrate(deps, env, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps<'_>, env: Env, msg: QueryMsg) -> Result<Binary> {
    contract::query(deps, env, msg)
}
