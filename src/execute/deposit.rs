use crate::{
    error::ContractError,
    math::{add_u64, mul_ratio_u128, sub_u128},
    msg::DepositMsg,
    state::{
        models::RevenueEvent,
        storage::{DELEGATION, EVENT_SEQ_NO, REVENUE_EVENTS, TAX_PCT},
    },
};
use cosmwasm_std::{attr, Response};

use super::Context;

pub fn exec_deposit(
    ctx: Context,
    params: DepositMsg,
) -> Result<Response, ContractError> {
    let Context { deps, env, .. } = ctx;
    let DepositMsg { amount: revenue } = params;
    let t = env.block.time;

    // Compute revenue shares for stakers and tax recipients
    let tax_pct = TAX_PCT.load(deps.storage)?;
    let tax_revenue = mul_ratio_u128(revenue, tax_pct, 1_000_000u128)?;
    let staking_revenue = sub_u128(revenue, tax_revenue)?;

    // Load total delegation amount across all delegators at this moment
    let total_delegation = DELEGATION.load(deps.storage)?;

    // Post-increment the sequence number
    let seq_no = EVENT_SEQ_NO
        .update(deps.storage, |n| -> Result<_, ContractError> {
            add_u64(n, 1u64)
        })?
        .u64()
        - 1;

    // insert revenue time series
    REVENUE_EVENTS.save(
        deps.storage,
        (t.nanos(), seq_no),
        &RevenueEvent {
            r: staking_revenue,
            d: total_delegation,
        },
    )?;

    Ok(Response::new().add_attributes(vec![attr("action", "deposit")]))
}
