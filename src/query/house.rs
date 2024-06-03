use cosmwasm_std::{Addr, Order};

use crate::{
    error::ContractError,
    responses::HouseResponse,
    state::storage::{
        BALANCES, CONFIG_FEE_RATE, CREATED_AT, CREATED_BY, DELEGATION, DESCRIPTION, SEQ_NO,
        FEES, NAME, N_ACCOUNTS, CONFIG_STAKE_TOKEN,
    },
    token::TokenAmount,
};

use super::ReadonlyContext;

pub fn query_house(ctx: ReadonlyContext) -> Result<HouseResponse, ContractError> {
    let ReadonlyContext { deps, .. } = ctx;
    return Ok(HouseResponse {
        name: NAME.may_load(deps.storage)?,
        description: DESCRIPTION.may_load(deps.storage)?,
        created_at: CREATED_AT.load(deps.storage)?,
        created_by: CREATED_BY.load(deps.storage)?,
        tax_pct: CONFIG_FEE_RATE.load(deps.storage)?,
        seq_no: SEQ_NO.load(deps.storage)?,
        n_accounts: N_ACCOUNTS.load(deps.storage)?,
        delegation: TokenAmount {
            token: CONFIG_STAKE_TOKEN.load(deps.storage)?,
            amount: DELEGATION.load(deps.storage)?,
        },
        balances: BALANCES
            .range(deps.storage, None, None, Order::Ascending)
            .map(|r| r.unwrap().1)
            .collect(),
        taxes: FEES
            .range(deps.storage, None, None, Order::Ascending)
            .map(|r| r.unwrap().1)
            .collect(),
    });
}
