use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128, Uint64};

use crate::token::Token;

#[cw_serde]
pub struct InstantiateMsg {
    pub name: Option<String>,
    pub description: Option<String>,
    pub stake_token: Token,
    pub liquidity_tokens: Vec<Token>,
    pub unbonding_seconds: Uint64,
    pub fee_rate: Uint128,
}

#[cw_serde]
pub struct StakeMsg {
    pub amount: Uint128,
    pub address: Option<Addr>,
}

#[cw_serde]
pub struct UnstakeMsg {
    pub amount: Option<Uint128>,
    pub address: Option<Addr>,
}

#[cw_serde]
pub struct DepositMsg {
    pub amount: Uint128,
    pub token: Token,
}

#[cw_serde]
pub struct ClaimMsg {
    pub token: Token,
}

#[cw_serde]
pub enum ExecuteMsg {
    Deposit(DepositMsg),
    Stake(StakeMsg),
}

#[cw_serde]
pub enum QueryMsg {
    Account { address: Addr },
    House {},
}

#[cw_serde]
pub struct MigrateMsg {}
