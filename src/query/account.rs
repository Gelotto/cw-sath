use cosmwasm_std::{Addr, Order, StdResult, Uint128};

use crate::{
    error::ContractError,
    responses::AccountResponse,
    state::{
        models::AccountSyncState,
        storage::{ACCOUNTS, ACCOUNT_SYNC_INFOS, ACCOUNT_UNBONDINGS, BALANCES, SEQ_NO},
    },
    sync::sync_account_balance,
    token::{Token, TokenAmount},
};

use super::ReadonlyContext;

pub fn query_account(
    ctx: ReadonlyContext,
    address: Addr,
) -> Result<Option<AccountResponse>, ContractError> {
    let ReadonlyContext { deps, .. } = ctx;
    let seq_no = SEQ_NO.load(deps.storage)?;

    if let Some(account) = ACCOUNTS.may_load(deps.storage, &address)? {
        let mut balances: Vec<TokenAmount> = Vec::with_capacity(2);

        for result in BALANCES
            .keys(deps.storage, None, None, Order::Ascending)
            .collect::<Vec<StdResult<_>>>()
        {
            let token_key = result?;
            let token = Token::from_key(&token_key);
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
            unbonding: ACCOUNT_UNBONDINGS.may_load(deps.storage, &address)?,
            balances,
        }));
    }

    Ok(None)
}
