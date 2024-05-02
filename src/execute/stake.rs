use crate::{
    error::ContractError,
    math::add_u128,
    msg::StakeMsg,
    state::{
        models::{Account, DelegationEvent},
        storage::{ACCOUNTS, DELEGATION, DELEGATION_EVENTS, EVENT_SEQ_NO},
    },
    sync::sync_account,
};
use cosmwasm_std::{attr, Response};

use super::Context;

pub fn exec_stake(
    ctx: Context,
    params: StakeMsg,
) -> Result<Response, ContractError> {
    let Context { deps, env, info } = ctx;
    let StakeMsg { amount, recipient } = params;
    let seq_no = EVENT_SEQ_NO.load(deps.storage)?;
    let t = env.block.time;

    // Stake on behalf of any specified recipient or default to tx sender
    let delegator = recipient.unwrap_or(info.sender);

    // Get or create delegator's account
    let mut account = ACCOUNTS
        .may_load(deps.storage, &delegator)?
        .unwrap_or_else(|| Account::new(t, seq_no));

    // Eagerly sync account before adding new delegation
    sync_account(deps.storage, deps.api, &mut account, &delegator, t, seq_no)?;

    account.add_delegation(amount)?;

    // Save account now that it has been synced and delegation incremented
    ACCOUNTS.save(deps.storage, &delegator, &account)?;

    // Increment total delegation across all accounts
    DELEGATION.update(deps.storage, |delegation| -> Result<_, ContractError> {
        add_u128(delegation, amount)
    })?;

    // Upsert a delegation event for this delegator
    DELEGATION_EVENTS.update(
        deps.storage,
        (&delegator, t.nanos(), seq_no.u64()),
        |maybe_event| -> Result<_, ContractError> {
            if let Some(mut event) = maybe_event {
                event.d = add_u128(event.d, amount)?;
                Ok(event)
            } else {
                Ok(DelegationEvent {
                    d: account.delegation,
                })
            }
        },
    )?;

    Ok(Response::new().add_attributes(vec![attr("action", "stake")]))
}
