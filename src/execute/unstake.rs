use crate::{
    error::ContractError,
    math::{add_u128, add_u64, div_u128, mul_ratio_u128, sub_u128},
    msg::UnstakeMsg,
    state::{
        models::{AccountUnbondingState, StakingEvent},
        storage::{
            ACCOUNTS, ACCOUNT_UNBONDINGS, CONFIG_UNBONDING_SECONDS, DELEGATION, SEQ_NO,
            STAKING_EVENTS, X,
        },
    },
    sync::{amortize, persist_sync_results, sync_account},
};
use cosmwasm_std::{attr, Response, Timestamp};

use super::Context;

pub fn exec_unstake(
    ctx: Context,
    params: UnstakeMsg,
) -> Result<Response, ContractError> {
    let Context { deps, env, info } = ctx;
    let UnstakeMsg {
        amount: maybe_amount,
        address,
    } = params;
    let seq_no = SEQ_NO.load(deps.storage)?;
    let t = env.block.time;

    // Stake on behalf of any specified recipient or default to tx sender
    let delegator_addr = address.unwrap_or(info.sender.to_owned());

    // Get or create delegator's account
    if let Some(mut account) = ACCOUNTS.may_load(deps.storage, &delegator_addr)? {
        let amount = maybe_amount.unwrap_or(account.delegation);

        // Eagerly sync account before adding new delegation
        let results = sync_account(
            deps.storage,
            deps.api,
            &delegator_addr,
            &account,
            seq_no,
            None,
            false,
        )?;

        for (result, state) in results.iter() {
            persist_sync_results(deps.storage, &info.sender, result, state)?;
        }

        // Decrement delegation amount
        account.subtract_delegation(amount)?;

        ACCOUNTS.save(deps.storage, &delegator_addr, &account)?;

        // Decrement total delegation across all accounts
        DELEGATION.update(deps.storage, |delegation| -> Result<_, ContractError> {
            sub_u128(delegation, amount)
        })?;

        // Upsert a delegation event for this delegator
        STAKING_EVENTS.update(
            deps.storage,
            (&delegator_addr, seq_no.u64()),
            |maybe_event| -> Result<_, ContractError> {
                if let Some(mut event) = maybe_event {
                    event.delta = sub_u128(event.delta, amount)?;
                    Ok(event)
                } else {
                    Ok(StakingEvent {
                        delta: account.delegation,
                    })
                }
            },
        )?;

        let duration_seconds: u64 = CONFIG_UNBONDING_SECONDS.load(deps.storage)?.into();

        ACCOUNT_UNBONDINGS.update(
            deps.storage,
            &delegator_addr,
            |maybe_info| -> Result<_, ContractError> {
                Ok(if let Some(mut info) = maybe_info {
                    let total = add_u128(info.amount, amount)?;
                    let new_ends_at = Timestamp::from_seconds(
                        div_u128(
                            add_u128(
                                mul_ratio_u128(
                                    info.unbonds_at.seconds() as u128,
                                    info.amount,
                                    total,
                                )?,
                                mul_ratio_u128(
                                    (env.block.time.seconds() + duration_seconds) as u128,
                                    amount,
                                    total,
                                )?,
                            )?,
                            2u128,
                        )?
                        .u128() as u64,
                    );
                    info.amount = total;
                    info.unbonds_at = new_ends_at;
                    info
                } else {
                    AccountUnbondingState {
                        amount: amount.to_owned(),
                        unbonds_at: env.block.time.plus_seconds(duration_seconds),
                    }
                })
            },
        )?;
    } else {
        return Err(ContractError::NotAuthorized {
            reason: "Account not found".to_owned(),
        });
    }

    // Increment trigger to indicate that next deposit should create new event.
    X.update(deps.storage, |x| -> Result<_, ContractError> {
        add_u64(x, 1u64)
    })?;

    amortize(deps.storage, seq_no, 5, None, Some(delegator_addr))?;

    Ok(Response::new().add_attributes(vec![attr("action", "stake")]))
}
