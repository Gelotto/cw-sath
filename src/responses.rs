use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp, Uint128};

use crate::{
    msg::HouseMarketingInfo,
    state::models::{
        AccountUnbondingState, DepositTokenAmount, Depositor, TaxRecipientBalance,
        TaxRecipientConfig, TaxRecipientInfo,
    },
    token::{Token, TokenAmount},
};

#[cw_serde]
pub struct AccountResponse {
    pub created_at: Timestamp,
    pub delegation: Uint128,
    pub balances: Vec<TokenAmount>,
    pub unbonding: Option<AccountUnbondingState>,
}

#[cw_serde]
pub struct HouseResponse {
    pub created_at: Timestamp,
    pub created_by: Addr,
    pub marketing: HouseMarketingInfo,
    pub delegation: TokenAmount,
    pub balances: Vec<TokenAmount>,
    pub stats: HouseStats,
}

#[cw_serde]
pub struct DepositsResponse {
    pub totals: Vec<DepositTokenAmount>,
    pub depositors: Vec<Depositor>,
}

#[cw_serde]
pub struct TaxRecipientResponseItem {
    pub address: Addr,
    pub info: TaxRecipientInfo,
    pub config: TaxRecipientConfig,
    pub totals: Vec<TaxRecipientBalance>,
}
#[cw_serde]
pub struct TaxesResponse {
    pub pct: Uint128,
    pub recipients: Vec<TaxRecipientResponseItem>,
}

#[cw_serde]
pub struct HouseStats {
    pub n_accounts: u32,
}

#[cw_serde]
pub struct BalanceEventCount {
    pub n: u32,
    pub token: Token,
}
