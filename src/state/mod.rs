pub mod models;
pub mod storage;

use cosmwasm_std::{Response, Uint128, Uint64};
use storage::{
    CONFIG_LIQUIDITY_TOKENS, CONFIG_STAKE_TOKEN, CONFIG_UNBONDING_SECONDS, CREATED_AT, CREATED_BY,
    DESCRIPTION, MANAGED_BY, NAME, N_ACCOUNTS,
};

use crate::{error::ContractError, execute::Context, msg::InstantiateMsg};

use self::storage::{CONFIG_FEE_RATE, DELEGATION, SEQ_NO};

/// Top-level initialization of contract state
pub fn init(
    ctx: Context,
    msg: &InstantiateMsg,
) -> Result<Response, ContractError> {
    let Context { deps, info, env } = ctx;
    DELEGATION.save(deps.storage, &Uint128::zero())?;
    SEQ_NO.save(deps.storage, &Uint64::zero())?;

    CREATED_AT.save(deps.storage, &env.block.time)?;
    CREATED_BY.save(deps.storage, &info.sender)?;
    MANAGED_BY.save(deps.storage, &info.sender)?;

    N_ACCOUNTS.save(deps.storage, &0)?;

    if let Some(name) = &msg.name {
        NAME.save(deps.storage, name)?;
    }

    if let Some(desc) = &msg.description {
        DESCRIPTION.save(deps.storage, desc)?;
    }

    CONFIG_STAKE_TOKEN.save(deps.storage, &msg.stake_token)?;
    CONFIG_UNBONDING_SECONDS.save(deps.storage, &msg.unbonding_seconds)?;
    CONFIG_FEE_RATE.save(deps.storage, &msg.fee_rate)?;

    for token in msg.liquidity_tokens.iter() {
        CONFIG_LIQUIDITY_TOKENS.save(deps.storage, &token.to_key(), token)?;
    }

    Ok(Response::new().add_attribute("action", "instantiate"))
}
