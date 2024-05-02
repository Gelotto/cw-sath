use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Timestamp, Uint128, Uint64};

use crate::{error::ContractError, math::add_u128};

#[cw_serde]
pub struct Config {}

#[cw_serde]
pub struct RevenueEvent {
    /// Revenue received
    pub r: Uint128,
    /// Total delegation at event time
    pub d: Uint128,
}

#[cw_serde]
pub struct DelegationEvent {
    /// Delegation increment received at time of event
    pub d: Uint128,
}

#[cw_serde]
pub struct DelegatorSyncState {
    pub t: Timestamp,
    pub seq_no: Uint64,
    pub amount: Uint128,
}

#[cw_serde]
pub struct Account {
    pub created_at: Timestamp,
    pub delegation: Uint128,
    pub sync: DelegatorSyncState,
}

impl DelegationEvent {
    pub fn default() -> Self {
        Self { d: Uint128::zero() }
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
            sync: DelegatorSyncState {
                t: time,
                seq_no,
                amount: Uint128::zero(),
            },
        }
    }

    pub fn add_delegation(
        &mut self,
        delta: Uint128,
    ) -> Result<Uint128, ContractError> {
        self.delegation = add_u128(self.delegation, delta)?;
        Ok(self.delegation)
    }
}
