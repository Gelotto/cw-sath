use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Timestamp, Uint128};

#[cw_serde]
pub struct AccountResponse {
    pub created_at: Timestamp,
    pub delegation: Uint128,
    pub balance: Uint128,
}
