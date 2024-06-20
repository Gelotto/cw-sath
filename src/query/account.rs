use cosmwasm_std::{Addr, Order, StdResult, Uint128, Uint64};

use crate::{
    error::ContractError,
    responses::AccountResponse,
    state::{
        models::AccountSyncState,
        storage::{ACCOUNTS, ACCOUNT_SYNC_INFOS, CONFIG_LIQUIDITY_TOKENS, SEQ_NO},
    },
    sync::sync_account_balance,
    token::{Token, TokenAmount},
};

use super::ReadonlyContext;

pub fn query_account(
    ctx: ReadonlyContext,
    address: Addr,
) -> Result<Option<AccountResponse>, ContractError> {
    let ReadonlyContext { deps, env, .. } = ctx;
    let seq_no = SEQ_NO.load(deps.storage)?;
    let t = env.block.time;

    if let Some(account) = ACCOUNTS.may_load(deps.storage, &address)? {
        if account.delegation.is_zero() {
            return Ok(None);
        }

        let mut balances: Vec<TokenAmount> = Vec::with_capacity(2);

        for result in CONFIG_LIQUIDITY_TOKENS
            .range(deps.storage, None, None, Order::Ascending)
            .collect::<Vec<StdResult<(_, Token)>>>()
        {
            let (_, token) = result?;
            let mut sync_state = ACCOUNT_SYNC_INFOS
                .may_load(deps.storage, (&address, &token.to_key()))?
                .or_else(|| {
                    Some(AccountSyncState {
                        amount: Uint128::zero(),
                        seq_no: account.created_at_seq_no,
                        t: account.created_at,
                    })
                })
                .unwrap();

            sync_account_balance(
                deps.storage,
                deps.api,
                &address,
                &mut sync_state,
                &token,
                seq_no,
                true,
            )?;

            balances.push(TokenAmount {
                amount: sync_state.amount,
                token,
            })
        }

        return Ok(Some(AccountResponse {
            created_at: account.created_at,
            delegation: account.delegation,
            balances,
        }));
    }

    Ok(None)
}
