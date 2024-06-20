use crate::{
    error::ContractError,
    math::{add_u128, add_u64, mul_ratio_u128, sub_u128},
    msg::DepositMsg,
    state::{
        models::BalanceEvent,
        storage::{
            BALANCES, CONFIG_FEE_RATE, DELEGATION, FEES, N_ACCOUNTS, N_DEPOSITS, SEQ_NO,
            TS_BALANCE, X,
        },
    },
    sync::amortize,
    token::TokenAmount,
};
use cosmwasm_std::{attr, Response, Uint64};

use super::Context;

pub fn exec_deposit(
    ctx: Context,
    params: DepositMsg,
) -> Result<Response, ContractError> {
    let Context { deps, .. } = ctx;
    // let t = env.block.time;
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
    let x = X.load(deps.storage)?;

    // Insert revenue time series
    let n_accounts = N_ACCOUNTS.load(deps.storage)?;

    //Increment n_deposits and return pre-incremented count
    let n_deposits = N_DEPOSITS.update(deps.storage, |n| -> Result<_, ContractError> {
        add_u64(n, 1u64)
    })? - Uint64::one();

    let seq_no = SEQ_NO.load(deps.storage)?;
    let mut insert_new_event = true;

    if !n_deposits.is_zero() {
        let key = (&token_key, seq_no.u64() - 1);
        let mut latest_event = TS_BALANCE.load(deps.storage, key)?;
        if latest_event.x == x {
            latest_event.delta = add_u128(latest_event.delta, revenue)?;
            TS_BALANCE.save(deps.storage, key, &latest_event)?;
            insert_new_event = false;
        }
    }
    if insert_new_event {
        // NOTE: The time series key uses the pre-incremented seq_no
        let key = (&token_key, seq_no.u64());
        SEQ_NO.save(deps.storage, &add_u64(seq_no, 1u64)?)?;
        TS_BALANCE.save(
            deps.storage,
            key,
            &BalanceEvent {
                delta: staking_revenue,
                total: total_delegation,
                ref_count: n_accounts,
                n_accounts,
                x,
            },
        )?;
    }

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

    amortize(deps.storage, seq_no.into(), 5, Some(token), None)?;

    Ok(Response::new().add_attributes(vec![attr("action", "deposit")]))
}
