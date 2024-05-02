use cosmwasm_std::{Addr, Uint128, Uint64};
use cw_storage_plus::{Item, Map};

use super::models::{Account, Config, DelegationEvent, RevenueEvent};

pub const CONFIG: Item<Config> = Item::new("config");
pub const DELEGATION: Item<Uint128> = Item::new("delegation");
pub const EVENT_SEQ_NO: Item<Uint64> = Item::new("event_seq_no");
pub const STAKING_REVENUE: Item<Uint128> = Item::new("staking_revenue");
pub const TAX_REVENUE: Item<Uint128> = Item::new("tax_revenue");
pub const TAX_PCT: Item<Uint128> = Item::new("tax_pct");

pub const ACCOUNTS: Map<&Addr, Account> = Map::new("accounts");
pub const REVENUE_EVENTS: Map<(u64, u64), RevenueEvent> = Map::new("ts_revenue");
pub const DELEGATION_EVENTS: Map<(&Addr, u64, u64), DelegationEvent> = Map::new("ts_deleg");
