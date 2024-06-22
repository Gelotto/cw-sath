use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128, Uint64};

use crate::token::Token;

#[cw_serde]
pub struct InstantiateMsg {
    pub marketing: HouseMarketingInfo,
    pub taxes: Vec<TaxRecipientInitArgs>,
    pub staking: StakingConfig,
}

#[cw_serde]
pub struct StakingConfig {
    pub staking_token: Token,
    pub revenue_tokens: Vec<Token>,
    pub min_increment: Option<Uint128>,
    pub unbonding_seconds: Option<Uint64>,
}

#[cw_serde]
pub struct TaxRecipientInitArgs {
    pub address: Addr,
    pub name: Option<String>,
    pub logo: Option<String>,
    pub pct: Uint128,
    pub autosend: bool,
    pub immutable: bool,
}

#[cw_serde]
pub struct HouseMarketingInfo {
    pub logo: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
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
    Unstake(UnstakeMsg),
    Claim(ClaimMsg),
}

#[cw_serde]
pub enum QueryMsg {
    Account { address: Addr },
    House {},
    Deposits {},
    Taxes {},
}

#[cw_serde]
pub struct MigrateMsg {}
