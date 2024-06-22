use cosmwasm_std::Order;

use crate::{
    error::ContractError,
    responses::{HouseResponse, HouseStats},
    state::storage::{
        BALANCES, CREATED_AT, CREATED_BY, MARKETING_INFO, STAKING_TOKEN, N_ACCOUNTS,
        TOTAL_DELEGATION,
    },
    token::TokenAmount,
};

use super::ReadonlyContext;

pub fn query_house(ctx: ReadonlyContext) -> Result<HouseResponse, ContractError> {
    let ReadonlyContext { deps, .. } = ctx;
    return Ok(HouseResponse {
        created_at: CREATED_AT.load(deps.storage)?,
        created_by: CREATED_BY.load(deps.storage)?,
        marketing: MARKETING_INFO.load(deps.storage)?,
        delegation: TokenAmount {
            token: STAKING_TOKEN.load(deps.storage)?,
            amount: TOTAL_DELEGATION.load(deps.storage)?,
        },
        balances: BALANCES
            .range(deps.storage, None, None, Order::Ascending)
            .map(|r| r.unwrap().1)
            .collect(),
        stats: HouseStats {
            n_accounts: N_ACCOUNTS.load(deps.storage)?,
        },
    });
}
