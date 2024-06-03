use cosmwasm_std::{Addr, Timestamp, Uint128, Uint64};
use cw_storage_plus::{Deque, Item, Map};

use crate::token::{Token, TokenAmount};

use super::models::{
    Account, AccountSyncState, AccountUnbondingState, BalanceEvent, Config, StakingEvent,
};

pub const CONFIG: Item<Config> = Item::new("config");
pub const CONFIG_UNBONDING_SECONDS: Item<Uint64> = Item::new("unbonding_seconds");
pub const CONFIG_STAKE_TOKEN: Item<Token> = Item::new("stake_wtoken");
pub const CONFIG_LIQUIDITY_TOKENS: Map<&String, Token> = Map::new("liquidity_tokens");
pub const CONFIG_FEE_RATE: Item<Uint128> = Item::new("fee_pct");

pub const CREATED_AT: Item<Timestamp> = Item::new("created_at");
pub const CREATED_BY: Item<Addr> = Item::new("created_by");
pub const MANAGED_BY: Item<Addr> = Item::new("managed_by");

pub const NAME: Item<String> = Item::new("name");
pub const DESCRIPTION: Item<String> = Item::new("description");
pub const DELEGATION: Item<Uint128> = Item::new("delegation");

pub const N_ACCOUNTS: Item<u32> = Item::new("n_accounts");

pub const ACCOUNTS: Map<&Addr, Account> = Map::new("accounts");
pub const ACCOUNT_SYNC_INFOS: Map<(&Addr, &String), AccountSyncState> = Map::new("account_syncs");
pub const ACCOUNT_UNBONDINGS: Map<&Addr, AccountUnbondingState> = Map::new("account_unbondings");

pub const SEQ_NO: Item<Uint64> = Item::new("seq_no");
pub const BALANCE_EVENTS: Map<(&String, u64, u64), BalanceEvent> = Map::new("ts_balance");
pub const STAKING_EVENTS: Map<(&Addr, u64, u64), StakingEvent> = Map::new("ts_stake");

pub const FEES: Map<&String, TokenAmount> = Map::new("fees");
pub const BALANCES: Map<&String, TokenAmount> = Map::new("balances");

pub const QUEUE: Deque<Addr> = Deque::new("queue");
