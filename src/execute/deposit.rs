use crate::{
    error::ContractError,
    math::{add_u128, add_u32, add_u64, mul_ratio_u128, sub_u128, sum_u128},
    msg::DepositMsg,
    state::{
        models::{BalanceEvent, DepositTotals, TaxRecipientBalance},
        storage::{
            BALANCES, DEPOSITOR_TOTALS, DEPOSIT_AGG_TOTALS, N_ACCOUNTS, N_BALANCE_EVENTS,
            N_DEPOSITS, REVENUE_TOKEN_KEYS, SEQ_NO, STAKING_TOKEN, TAX_RECIPIENT_CONFIGS,
            TAX_RECIPIENT_TOTALS, TAX_TOTAL_BALANCES, TOTAL_DELEGATION, TOTAL_UNBONDING,
            TS_BALANCE, X,
        },
    },
    sync::amortize,
    token::TokenAmount,
};
use cosmwasm_std::{
    attr, Addr, Empty, Order, QuerierWrapper, Response, StdResult, Storage, SubMsg, Uint128, Uint64,
};

use super::Context;

pub fn exec_deposit(
    ctx: Context,
    params: DepositMsg,
) -> Result<Response, ContractError> {
    let Context { deps, env, info } = ctx;
    let seq_no = SEQ_NO.load(deps.storage)?;
    let token_key = params.token.to_key();
    let mut resp = Response::new().add_attributes(vec![attr("action", "deposit")]);
    let mut params = params;

    // Only allow whitelisted token types
    if !REVENUE_TOKEN_KEYS.has(deps.storage, &token_key) {
        return Err(ContractError::NotAuthorized {
            reason: "token type not accepted in deposits".to_owned(),
        });
    }

    // Update the depostor's totals BEFORE syncging untracked balance which
    // changing params.amount in place.
    update_depositor_totals(deps.storage, &info.sender, &token_key, params.amount)?;

    // Sync any untracked balance with the tracked balance and add the
    // difference to the total amount to be deposited below
    params.amount = add_u128(
        sync_untracked_balance(deps.storage, deps.querier, &env.contract.address, &params)?,
        params.amount,
    )?;

    // Perform deposit and return submsg to transfer any tax to tax recipient
    let fee_transfer_submsgs = deposit(deps.storage, params.to_owned(), seq_no)?;
    resp = resp.add_submessages(fee_transfer_submsgs);

    amortize(
        deps.storage,
        deps.api,
        seq_no.into(),
        Some(params.token),
        None,
    )?;

    Ok(resp)
}

fn sync_untracked_balance(
    store: &mut dyn Storage,
    querier: QuerierWrapper<Empty>,
    contract_addr: &Addr,
    params: &DepositMsg,
) -> Result<Uint128, ContractError> {
    let DepositMsg { amount, token } = params.to_owned();
    let contract_balance = token.query_balance(querier, contract_addr)?;
    let token_key = &token.to_key();

    let tracked_balance = BALANCES
        .may_load(store, token_key)?
        .and_then(|b| Some(b.amount))
        .unwrap_or_default();

    let untracked_balance = sub_u128(
        contract_balance,
        if *token_key == STAKING_TOKEN.load(store)?.to_key() {
            sum_u128(vec![
                amount,
                TOTAL_DELEGATION.load(store)?,
                TOTAL_UNBONDING.load(store)?,
                TAX_TOTAL_BALANCES
                    .load(store, token_key)
                    .unwrap_or_default(),
            ])?
        } else {
            amount
        },
    )?;

    // Sync any untracked balance with the tracked balance
    if tracked_balance < untracked_balance {
        let delta = untracked_balance - tracked_balance;
        update_depositor_totals(store, contract_addr, token_key, delta)?;
        return Ok(delta);
    }

    Ok(Uint128::zero())
}

fn deposit(
    store: &mut dyn Storage,
    params: DepositMsg,
    seq_no: Uint64,
) -> Result<Vec<SubMsg>, ContractError> {
    let DepositMsg {
        amount: revenue,
        token,
    } = params;

    let token_key = token.to_key();

    // Send or allocate taxes to fee recipients
    let mut transfer_fee_submsgs: Vec<SubMsg> = Vec::with_capacity(1);
    let mut tax_revenue = Uint128::zero();

    for result in TAX_RECIPIENT_CONFIGS
        .range(store, None, None, Order::Ascending)
        .collect::<Vec<StdResult<_>>>()
    {
        let (tax_recipient_addr, info) = result?;
        let tax_delta = mul_ratio_u128(revenue, info.pct, 1_000_000u128)?;

        tax_revenue = add_u128(tax_revenue, tax_delta)?;

        if info.autosend {
            transfer_fee_submsgs.push(token.transfer(&tax_recipient_addr, tax_delta)?);
        } else {
            // Increment total amoutn held for taxes with respect to this token type
            TAX_TOTAL_BALANCES.update(store, &token_key, |n| -> Result<_, ContractError> {
                add_u128(n.unwrap_or_default(), tax_delta)
            })?;
        }

        TAX_RECIPIENT_TOTALS.update(
            store,
            (&tax_recipient_addr, &token_key),
            |maybe_totals| -> Result<_, ContractError> {
                let mut totals = maybe_totals.unwrap_or_else(|| TaxRecipientBalance {
                    balance: Uint128::zero(),
                    total: Uint128::zero(),
                });
                totals.total = add_u128(totals.total, tax_delta)?;
                if !info.autosend {
                    totals.balance = add_u128(totals.balance, tax_delta)?;
                }
                Ok(totals)
            },
        )?;
    }

    // Compute house revenue after taxes
    let staking_revenue = sub_u128(revenue, tax_revenue)?;

    // Load total delegation amount across all delegators at this moment
    let total_delegation = TOTAL_DELEGATION.load(store)?;
    let x = X.load(store)?;

    // Insert revenue time series
    let n_accounts = N_ACCOUNTS.load(store)?;

    //Increment n_deposits and return pre-incremented count
    N_DEPOSITS.update(store, &token_key, |maybe_n| -> Result<_, ContractError> {
        add_u64(maybe_n.unwrap_or_default(), 1u64)
    })?;

    let mut insert_new_event = true;

    // If applicable, update the most recent "balance" time series entry.
    // otherwise, create a new one below.
    if N_BALANCE_EVENTS
        .may_load(store, &token_key)?
        .unwrap_or_default()
        > 0
    {
        let key = (&token_key, seq_no.u64() - 1);
        let mut existing_event = TS_BALANCE.load(store, key)?;
        if existing_event.x == x {
            existing_event.delta = add_u128(existing_event.delta, revenue)?;
            TS_BALANCE.save(store, key, &existing_event)?;
            insert_new_event = false;
        }
    }

    // Create a new "balance" timeseries entry
    if insert_new_event {
        // NOTE: The time series key uses the pre-incremented seq_no
        SEQ_NO.save(store, &add_u64(seq_no, 1u64)?)?;

        TS_BALANCE.save(
            store,
            (&token_key, seq_no.u64()),
            &BalanceEvent {
                delta: staking_revenue,
                total: total_delegation,
                ref_count: n_accounts,
                n_accounts,
                x,
            },
        )?;

        N_BALANCE_EVENTS.update(store, &token_key, |maybe_n| -> Result<_, ContractError> {
            add_u32(maybe_n.unwrap_or_default(), 1)
        })?;
    }

    // Increment running total historical deposit amount
    DEPOSIT_AGG_TOTALS.update(
        store,
        &token_key,
        |maybe_totals| -> Result<_, ContractError> {
            let mut totals = maybe_totals.unwrap_or_else(|| DepositTotals {
                amount: Uint128::zero(),
                n: Uint64::zero(),
            });
            totals.amount = add_u128(totals.amount, params.amount)?;
            totals.n = add_u64(totals.n, 1u64)?;
            Ok(totals)
        },
    )?;

    // Increment global house revenue for this token type
    BALANCES.update(store, &token_key, |maybe_ta| -> Result<_, ContractError> {
        Ok(if let Some(mut ta) = maybe_ta {
            ta.amount += revenue;
            ta
        } else {
            TokenAmount {
                amount: revenue,
                token: token.to_owned(),
            }
        })
    })?;

    Ok(transfer_fee_submsgs)
}

fn update_depositor_totals(
    store: &mut dyn Storage,
    address: &Addr,
    token_key: &String,
    amount: Uint128,
) -> Result<(), ContractError> {
    DEPOSITOR_TOTALS.update(
        store,
        (token_key, address),
        |maybe_totals| -> Result<_, ContractError> {
            let mut totals = maybe_totals.unwrap_or_else(|| DepositTotals {
                amount: Uint128::zero(),
                n: Uint64::zero(),
            });
            totals.amount = add_u128(totals.amount, amount)?;
            totals.n = add_u64(totals.n, 1u64)?;
            Ok(totals)
        },
    )?;

    Ok(())
}
