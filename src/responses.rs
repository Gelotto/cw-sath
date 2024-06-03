use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp, Uint128, Uint64};

use crate::token::TokenAmount;

#[cw_serde]
pub struct AccountResponse {
    pub created_at: Timestamp,
    pub delegation: Uint128,
    pub balances: Vec<TokenAmount>,
}

#[cw_serde]
pub struct HouseResponse {
    pub name: Option<String>,
    pub description: Option<String>,
    pub created_at: Timestamp,
    pub created_by: Addr,
    pub delegation: TokenAmount,
    pub n_accounts: u32,
    pub seq_no: Uint64,
    pub balances: Vec<TokenAmount>,
    pub taxes: Vec<TokenAmount>,
    pub tax_pct: Uint128,
}
