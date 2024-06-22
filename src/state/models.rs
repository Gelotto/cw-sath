use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp, Uint128, Uint64};

use crate::{
    error::ContractError,
    math::{add_u128, sub_u128},
    token::{Token, TokenAmount},
};

#[cw_serde]
pub struct BalanceEvent {
    /// Amount received
    pub delta: Uint128,
    /// Total delegation at event time
    pub total: Uint128,
    /// Number of accounts
    pub n_accounts: u32,
    /// Reference count for garbage collection
    pub ref_count: u32,
    /// Value used to determine if new event is needed
    /// or if this one can simply be updated. Compared
    /// with the global X storage var.
    pub x: Uint64,
}

#[cw_serde]
pub struct StakingEvent {
    /// Delegation increment received at time of event
    pub delta: Uint128,
}

#[cw_serde]
pub struct AccountSyncState {
    pub t: Timestamp,
    pub seq_no: Uint64,
    pub amount: Uint128,
}

#[cw_serde]
pub struct DepositTotals {
    pub n: Uint64,
    pub amount: Uint128,
}

#[cw_serde]
pub struct DepositTokenAmount {
    pub n: Uint64,
    pub amount: Uint128,
    pub token: Token,
}

#[cw_serde]
pub struct TaxRecipientInfo {
    pub name: Option<String>,
    pub logo: Option<String>,
}

#[cw_serde]
pub struct TaxRecipientConfig {
    pub pct: Uint128,
    pub autosend: bool,
    pub immutable: bool,
}

#[cw_serde]
pub struct TaxRecipientBalance {
    pub balance: Uint128,
    pub total: Uint128,
}

#[cw_serde]
pub struct TaxRecipient {
    pub address: Addr,
    pub totals: Vec<TokenAmount>,
    pub name: Option<String>,
    pub auto: bool,
}

#[cw_serde]
pub struct Depositor {
    pub address: Addr,
    pub totals: Vec<DepositTokenAmount>,
}

#[cw_serde]
pub struct AccountUnbondingState {
    pub amount: Uint128,
    pub unbonds_at: Timestamp,
}

#[cw_serde]
pub struct Account {
    pub created_at: Timestamp,
    pub created_at_seq_no: Uint64,
    pub delegation: Uint128,
}

impl StakingEvent {
    pub fn default() -> Self {
        Self {
            delta: Uint128::zero(),
        }
    }
}

impl Account {
    pub fn new(
        time: Timestamp,
        seq_no: Uint64,
    ) -> Self {
        Self {
            created_at: time,
            delegation: Uint128::zero(),
            created_at_seq_no: seq_no,
        }
    }

    pub fn add_delegation(
        &mut self,
        delta: Uint128,
    ) -> Result<Uint128, ContractError> {
        self.delegation = add_u128(self.delegation, delta)?;
        Ok(self.delegation)
    }
    pub fn subtract_delegation(
        &mut self,
        delta: Uint128,
    ) -> Result<Uint128, ContractError> {
        self.delegation = sub_u128(self.delegation, delta)?;
        Ok(self.delegation)
    }
}
