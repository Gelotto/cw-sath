pub mod models;
pub mod storage;

use cosmwasm_std::{Response, Uint128, Uint64};

use crate::{error::ContractError, execute::Context, msg::InstantiateMsg};

use self::storage::{DELEGATION, EVENT_SEQ_NO, STAKING_REVENUE, TAX_PCT, TAX_REVENUE};

/// Top-level initialization of contract state
pub fn init(
    ctx: Context,
    _msg: &InstantiateMsg,
) -> Result<Response, ContractError> {
    let Context { deps, .. } = ctx;
    TAX_PCT.save(deps.storage, &Uint128::zero())?;
    DELEGATION.save(deps.storage, &Uint128::zero())?;
    STAKING_REVENUE.save(deps.storage, &Uint128::zero())?;
    TAX_REVENUE.save(deps.storage, &Uint128::zero())?;
    EVENT_SEQ_NO.save(deps.storage, &Uint64::zero())?;
    Ok(Response::new().add_attribute("action", "instantiate"))
}
