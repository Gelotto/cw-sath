use crate::{
    error::ContractError,
    math::add_u64,
    msg::ClaimMsg,
    state::storage::{ACCOUNTS, SEQ_NO, X},
    sync::{amortize, persist_sync_results, sync_account},
};
use cosmwasm_std::{attr, Response, SubMsg, Uint128, Uint64};

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
        transfer_submsgs.push(token.transfer(&info.sender, sync_state.amount)?);
        sync_state.amount = Uint128::zero();
        persist_sync_results(deps.storage, &info.sender, result, sync_state)?;
    }

    // Increment trigger to indicate that next deposit should create new event.
    X.update(deps.storage, |x| -> Result<_, ContractError> {
        add_u64(x, 1u64)
    })?;

    amortize(
        deps.storage,
        seq_no.into(),
        5,
        Some(params.token),
        Some(info.sender),
    )?;

    Ok(Response::new()
        .add_attributes(vec![attr("action", "claim")])
        .add_submessages(transfer_submsgs))
}
