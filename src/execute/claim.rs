use crate::{
    error::ContractError,
    math::{add_u64, sub_u128},
    msg::ClaimMsg,
    state::storage::{
        ACCOUNTS, ACCOUNT_UNBONDINGS, BALANCES, SEQ_NO, STAKING_TOKEN, TOTAL_UNBONDING, X,
    },
    sync::{amortize, persist_sync_results, sync_account},
};
use cosmwasm_std::{attr, Response, SubMsg, Uint128};

use super::Context;

pub fn exec_claim(
    ctx: Context,
    params: ClaimMsg,
) -> Result<Response, ContractError> {
    let Context { deps, info, env } = ctx;
    let seq_no = SEQ_NO.load(deps.storage)?;

    // sync the account before processing claim
    let mut sync_states = sync_account(
        deps.storage,
        deps.api,
        &info.sender,
        &ACCOUNTS.load(deps.storage, &info.sender)?,
        seq_no,
        Some(params.token.to_owned()),
        true,
    )?;

    // accumulate transfer submsgs and reset sync amounts to 0
    let mut transfer_submsgs: Vec<SubMsg> = Vec::with_capacity(sync_states.len());

    for (result, sync_state) in sync_states.iter_mut() {
        let token = &result.token;
        let token_key = token.to_key();

        transfer_submsgs.push(token.transfer(&info.sender, sync_state.amount)?);

        let updated_balance = BALANCES.update(
            deps.storage,
            &token_key,
            |maybe_b| -> Result<_, ContractError> {
                let mut b = maybe_b.unwrap(); // initialized in instantiate
                b.amount = sub_u128(b.amount, sync_state.amount)?;
                Ok(b)
            },
        )?;

        sync_state.amount = Uint128::zero();
        if updated_balance.amount.is_zero() {
            BALANCES.remove(deps.storage, &token_key);
        }

        persist_sync_results(deps.storage, &info.sender, result, sync_state)?;
    }

    // Increment trigger to indicate that next deposit should create new event.
    X.update(deps.storage, |x| -> Result<_, ContractError> {
        add_u64(x, 1u64)
    })?;

    // If the tx sender is unbonding and the unbonding timeout has ellapsed,
    // then send them their unbonded amount in addition to everything else.
    if let Some(unbonding) = ACCOUNT_UNBONDINGS.may_load(deps.storage, &info.sender)? {
        if env.block.time >= unbonding.unbonds_at {
            let staking_token = STAKING_TOKEN.load(deps.storage)?;
            transfer_submsgs.push(staking_token.transfer(&info.sender, unbonding.amount)?);
            ACCOUNT_UNBONDINGS.remove(deps.storage, &info.sender);
            TOTAL_UNBONDING.update(deps.storage, |n| -> Result<_, ContractError> {
                sub_u128(n, unbonding.amount)
            })?;
        }
    }

    amortize(
        deps.storage,
        deps.api,
        seq_no.into(),
        Some(params.token),
        Some(info.sender),
    )?;

    Ok(Response::new()
        .add_attributes(vec![attr("action", "claim")])
        .add_submessages(transfer_submsgs))
}
