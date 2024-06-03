use crate::{
    error::ContractError,
    math::{add_u128, add_u32},
    msg::StakeMsg,
    state::{
        models::{Account, StakingEvent},
        storage::{
            ACCOUNTS, CONFIG_STAKE_TOKEN, DELEGATION, N_ACCOUNTS, QUEUE, SEQ_NO, STAKING_EVENTS,
        },
    },
    sync::{amortize, persist_sync_results, sync_account},
};
use cosmwasm_std::{attr, Response};

use super::Context;

pub fn exec_stake(
    ctx: Context,
    params: StakeMsg,
) -> Result<Response, ContractError> {
    let Context { deps, env, info } = ctx;
    let seq_no = SEQ_NO.load(deps.storage)?;
    let t = env.block.time;
    let StakeMsg {
        amount,
        address: recipient,
    } = params;

    // Stake on behalf of any specified recipient or default to tx sender
    let token = CONFIG_STAKE_TOKEN.load(deps.storage)?;
    let staker = recipient.unwrap_or(info.sender.to_owned());

    // Get or create stake account
    let mut account = if let Some(account) = ACCOUNTS.may_load(deps.storage, &staker)? {
        let results = sync_account(
            deps.storage,
            deps.api,
            &staker,
            &account,
            t,
            seq_no,
            Some(token.to_owned()),
        )?;
        for (result, state) in results.iter() {
            persist_sync_results(deps.storage, &info.sender, result, state)?;
        }
        account
    } else {
        // Add to amortization queue since account is new
        QUEUE.push_back(deps.storage, &staker)?;
        N_ACCOUNTS.update(deps.storage, |n| -> Result<_, ContractError> {
            add_u32(n, 1)
        })?;
        Account::new(t, seq_no)
    };

    account.add_delegation(amount)?;

    // Save account now that it has been synced and delegation incremented
    ACCOUNTS.save(deps.storage, &staker, &account)?;

    // Increment total delegation across all accounts
    DELEGATION.update(deps.storage, |delegation| -> Result<_, ContractError> {
        add_u128(delegation, amount)
    })?;

    // Upsert a delegation event for this delegator
    STAKING_EVENTS.update(
        deps.storage,
        (&staker, t.nanos(), seq_no.u64()),
        |maybe_event| -> Result<_, ContractError> {
            if let Some(mut event) = maybe_event {
                event.delta = add_u128(event.delta, amount)?;
                Ok(event)
            } else {
                Ok(StakingEvent {
                    delta: account.delegation,
                })
            }
        },
    )?;

    amortize(
        deps.storage,
        t,
        seq_no,
        5,
        Some(token.to_owned()),
        Some(staker),
    )?;

    Ok(Response::new().add_attributes(vec![attr("action", "stake")]))
}
