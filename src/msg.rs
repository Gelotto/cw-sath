use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub struct StakeMsg {
    pub amount: Uint128,
    pub recipient: Option<Addr>,
}

#[cw_serde]
pub struct DepositMsg {
    pub amount: Uint128,
}

#[cw_serde]
pub enum ExecuteMsg {
    Deposit(DepositMsg),
    Stake(StakeMsg),
}

#[cw_serde]
pub enum QueryMsg {
    Account { address: Addr },
}

#[cw_serde]
pub struct MigrateMsg {}
