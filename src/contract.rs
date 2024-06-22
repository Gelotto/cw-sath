use crate::error::ContractError;
use crate::execute::claim::exec_claim;
use crate::execute::deposit::exec_deposit;
use crate::execute::stake::exec_stake;
use crate::execute::unstake::exec_unstake;
use crate::execute::Context;
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::query::account::query_account;
use crate::query::deposits::query_deposits;
use crate::query::house::query_house;
use crate::query::taxes::query_taxes;
use crate::query::ReadonlyContext;
use crate::state;
use cosmwasm_std::{entry_point, to_json_binary};
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;

const CONTRACT_NAME: &str = "crates.io:cw-sath";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(state::init(Context { deps, env, info }, msg)?)
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let ctx = Context { deps, env, info };

    match msg {
        ExecuteMsg::Deposit(msg) => exec_deposit(ctx, msg),
        ExecuteMsg::Stake(msg) => exec_stake(ctx, msg),
        ExecuteMsg::Unstake(msg) => exec_unstake(ctx, msg),
        ExecuteMsg::Claim(msg) => exec_claim(ctx, msg),
    }
}

#[entry_point]
pub fn query(
    deps: Deps,
    env: Env,
    msg: QueryMsg,
) -> Result<Binary, ContractError> {
    let ctx = ReadonlyContext { deps, env };
    let result = match msg {
        QueryMsg::Account { address } => to_json_binary(&query_account(ctx, address)?),
        QueryMsg::House {} => to_json_binary(&query_house(ctx)?),
        QueryMsg::Taxes {} => to_json_binary(&query_taxes(ctx)?),
        QueryMsg::Deposits {} => to_json_binary(&query_deposits(ctx)?),
    }?;
    Ok(result)
}

#[entry_point]
pub fn migrate(
    deps: DepsMut,
    _env: Env,
    _msg: MigrateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
