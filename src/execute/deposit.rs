use crate::{
    error::ContractError,
    math::{add_u64, mul_ratio_u128, sub_u128},
    msg::DepositMsg,
    state::{
        models::BalanceEvent,
        storage::{
            BALANCES, BALANCE_EVENTS, CONFIG_FEE_RATE, DELEGATION, FEES, N_ACCOUNTS, SEQ_NO,
        },
    },
    sync::amortize,
    token::TokenAmount,
};
use cosmwasm_std::{attr, Response};

use super::Context;

pub fn exec_deposit(
    ctx: Context,
    params: DepositMsg,
) -> Result<Response, ContractError> {
    let Context { deps, env, .. } = ctx;
    let t = env.block.time;
    let DepositMsg {
        amount: revenue,
        token,
    } = params;

    // Compute revenue shares for stakers and tax recipients
    let tax_pct = CONFIG_FEE_RATE.load(deps.storage)?;
    let tax_revenue = mul_ratio_u128(revenue, tax_pct, 1_000_000u128)?;
    let staking_revenue = if tax_pct.is_zero() {
        revenue
    } else {
        sub_u128(revenue, tax_revenue)?
    };

    let token_key = token.to_key();

    // Load total delegation amount across all delegators at this moment
    let total_delegation = DELEGATION.load(deps.storage)?;

    // Post-increment the sequence number
    let seq_no = SEQ_NO
        .update(deps.storage, |n| -> Result<_, ContractError> {
            add_u64(n, 1u64)
        })?
        .u64()
        - 1;

    // Insert revenue time series
    let n_accounts = N_ACCOUNTS.load(deps.storage)?;

    BALANCE_EVENTS.save(
        deps.storage,
        (&token_key, t.nanos(), seq_no),
        &BalanceEvent {
            delta: staking_revenue,
            total: total_delegation,
            n_accounts,
            ref_count: n_accounts,
        },
    )?;

    // Increment global house revenue for this token type
    BALANCES.update(
        deps.storage,
        &token_key,
        |maybe_ta| -> Result<_, ContractError> {
            Ok(if let Some(mut ta) = maybe_ta {
                ta.amount += revenue;
                ta
            } else {
                TokenAmount {
                    amount: revenue,
                    token: token.to_owned(),
                }
            })
        },
    )?;

    // Increment global tax revenue for this token type
    if !tax_revenue.is_zero() {
        FEES.update(
            deps.storage,
            &token_key,
            |maybe_ta| -> Result<_, ContractError> {
                Ok(if let Some(mut ta) = maybe_ta {
                    ta.amount += tax_revenue;
                    ta
                } else {
                    TokenAmount {
                        amount: tax_revenue,
                        token: token.to_owned(),
                    }
                })
            },
        )?;
    }

    amortize(deps.storage, t, seq_no.into(), 5, Some(token), None)?;

    Ok(Response::new().add_attributes(vec![attr("action", "deposit")]))
}
