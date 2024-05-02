use cosmwasm_std::{Addr, Uint128};

use crate::{
    error::ContractError,
    responses::AccountResponse,
    state::{
        models::Account,
        storage::{ACCOUNTS, EVENT_SEQ_NO},
    },
    sync::sync_account,
};

use super::ReadonlyContext;

pub fn query_account(
    ctx: ReadonlyContext,
    address: Addr,
) -> Result<AccountResponse, ContractError> {
    let ReadonlyContext { deps, env, .. } = ctx;
    let seq_no = EVENT_SEQ_NO.load(deps.storage)?;
    let t = env.block.time;

    let mut account = ACCOUNTS
        .may_load(deps.storage, &address)?
        .unwrap_or_else(|| Account::new(t, seq_no));

    if !account.delegation.is_zero() {
        sync_account(deps.storage, deps.api, &mut account, &address, t, seq_no)?;
    }

    Ok(AccountResponse {
        created_at: account.created_at,
        delegation: account.delegation,
        balance: account.sync.amount,
    })
}
