use crate::{
    error::ContractError,
    math::{add_u128, add_u64, mul_ratio_u128, sub_u128},
    msg::UnstakeMsg,
    state::{
        models::{AccountUnbondingState, StakingEvent},
        storage::{
            ACCOUNTS, ACCOUNT_UNBONDINGS, MANAGED_BY, SEQ_NO, TOTAL_DELEGATION, TOTAL_UNBONDING,
            TS_STAKE, UNBONDING_SECONDS, X,
        },
    },
    sync::{amortize, persist_sync_results, sync_account},
};
use cosmwasm_std::{attr, ensure_eq, Attribute, Response};

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

    // Unstake on behalf of any specified recipient or default to tx sender
    let account_addr = if let Some(account_addr) = address {
        ensure_eq!(
            account_addr,
            MANAGED_BY.load(deps.storage)?,
            ContractError::NotAuthorized {
                reason: "only the contract manager can unstake on behalf of another account"
                    .to_owned()
            }
        );
        account_addr
    } else {
        info.sender.to_owned()
    };

    let mut attrs: Vec<Attribute> = vec![attr("action", "unstake")];
    let seq_no = SEQ_NO.load(deps.storage)?;

    // Get or create delegator's account
    if let Some(mut account) = ACCOUNTS.may_load(deps.storage, &account_addr)? {
        let amount = maybe_amount.unwrap_or(account.delegation);

        // Eagerly sync account before adding new delegation
        let results = sync_account(
            deps.storage,
            deps.api,
            &account_addr,
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

        ACCOUNTS.save(deps.storage, &account_addr, &account)?;

        // Decrement total delegation across all accounts
        TOTAL_DELEGATION.update(deps.storage, |n| -> Result<_, ContractError> {
            sub_u128(n, amount)
        })?;

        // Increase total unbonding amount
        TOTAL_UNBONDING.update(deps.storage, |n| -> Result<_, ContractError> {
            add_u128(n, amount)
        })?;

        // Upsert a delegation event for this delegator
        TS_STAKE.update(
            deps.storage,
            (&account_addr, seq_no.u64()),
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

        let duration_seconds: u64 = UNBONDING_SECONDS.load(deps.storage)?.into();

        let unbonding = ACCOUNT_UNBONDINGS.update(
            deps.storage,
            &account_addr,
            |maybe_unbonding| -> Result<_, ContractError> {
                Ok(if let Some(mut unbonding) = maybe_unbonding {
                    let total = add_u128(unbonding.amount, amount)?;
                    let new_ends_at = unbonding.unbonds_at.plus_seconds(
                        mul_ratio_u128(duration_seconds as u128, amount, total)?
                            .u128()
                            .clamp(0u128, u64::MAX as u128) as u64,
                    );
                    unbonding.amount = total;
                    unbonding.unbonds_at = new_ends_at;
                    unbonding
                } else {
                    AccountUnbondingState {
                        amount: amount.to_owned(),
                        unbonds_at: env.block.time.plus_seconds(duration_seconds),
                    }
                })
            },
        )?;

        attrs.push(attr("unbonds_at", unbonding.unbonds_at.nanos().to_string()));
        attrs.push(attr("unbond_amount", unbonding.amount.u128().to_string()));
        attrs.push(attr("initiated_at", env.block.time.nanos().to_string()));
    } else {
        return Err(ContractError::NotAuthorized {
            reason: "Account not found".to_owned(),
        });
    }

    // Increment trigger to indicate that next deposit should create new event.
    X.update(deps.storage, |x| -> Result<_, ContractError> {
        add_u64(x, 1u64)
    })?;

    amortize(deps.storage, deps.api, seq_no, None, Some(account_addr))?;

    Ok(Response::new().add_attributes(attrs))
}
