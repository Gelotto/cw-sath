use cosmwasm_std::{Addr, Timestamp, Uint128, Uint64};
use cw_storage_plus::{Deque, Item, Map};

use crate::{
    msg::HouseMarketingInfo,
    token::{Token, TokenAmount},
};

use super::models::{
    Account, AccountSyncState, AccountUnbondingState, BalanceEvent, DepositTotals, StakingEvent,
    TaxRecipientBalance, TaxRecipientConfig, TaxRecipientInfo,
};

pub type TokenKey = String;

/// Minimum increment by which a user can increase their delegation by staking
pub const MIN_STAKE_INCREMENT: Item<Addr> = Item::new("min_increment");

/// Max timeout between unstaking and being able to claim unstaked delegation
pub const UNBONDING_SECONDS: Item<Uint64> = Item::new("unbonding_seconds");

/// Tax recipient metadata, like name & logo
pub const TAX_RECIPIENT_INFOS: Map<&Addr, TaxRecipientInfo> = Map::new("tax_recipient_infos");

/// Tax recipient settings used when computing and sending taxes
pub const TAX_RECIPIENT_CONFIGS: Map<&Addr, TaxRecipientConfig> = Map::new("tax_recipient_configs");

/// Total amount of taxes in held in contract for tax recipients, pending claim
pub const TAX_TOTAL_BALANCES: Map<&TokenKey, Uint128> = Map::new("tax_total_balances");

/// Total amount of taxes received and amount pending claim for each recipient
pub const TAX_RECIPIENT_TOTALS: Map<(&Addr, &TokenKey), TaxRecipientBalance> =
    Map::new("tax_recipient_totals");

/// Grand total amount of delegated (not unbonding) token across all stakers
pub const TOTAL_DELEGATION: Item<Uint128> = Item::new("total_delegation");

/// Grand total amount of unbonding token
pub const TOTAL_UNBONDING: Item<Uint128> = Item::new("total_unbonding");

/// Total number of staked accounts
pub const N_ACCOUNTS: Item<u32> = Item::new("n_accounts");

/// Current existent nubmer of entries in TS_BALANCE for a given token type
pub const N_BALANCE_EVENTS: Map<&TokenKey, u32> = Map::new("n_balance_events");

/// Token type used for staking
pub const STAKING_TOKEN: Item<Token> = Item::new("stake_wtoken");

/// Token types accepted in deposits, i.e. revenue for stakers
pub const REVENUE_TOKEN_KEYS: Map<&String, u8> = Map::new("revenue_token_keys");

/// Marketing info like name, description, etc. for this house
pub const MARKETING_INFO: Item<HouseMarketingInfo> = Item::new("marketing_info");

/// Contract creation timestamp
pub const CREATED_AT: Item<Timestamp> = Item::new("created_at");

/// Address of creator
pub const CREATED_BY: Item<Addr> = Item::new("created_by");

/// Address of manager contract or wallet
pub const MANAGED_BY: Item<Addr> = Item::new("managed_by");

/// Aggregate total number of deposits per token type
pub const N_DEPOSITS: Map<&TokenKey, Uint64> = Map::new("n_deposits");

/// Total amount deposited for each token type
pub const DEPOSIT_AGG_TOTALS: Map<&TokenKey, DepositTotals> = Map::new("deposit_agg_totals");

/// Total amount deposited by each depositor for each token type
pub const DEPOSITOR_TOTALS: Map<(&TokenKey, &Addr), DepositTotals> = Map::new("depositor_totals");

/// A value that is used to determine whether a new TS_BALANCE entry should be
/// created instead of updating the latest entry.
pub const X: Item<Uint64> = Item::new("x");

/// Storage for staking accounts
pub const ACCOUNTS: Map<&Addr, Account> = Map::new("accounts");

/// State that pertains to the token balances of each staker
pub const ACCOUNT_SYNC_INFOS: Map<(&Addr, &TokenKey), AccountSyncState> = Map::new("account_syncs");

/// Storage for an account while unbonding via unstake
pub const ACCOUNT_UNBONDINGS: Map<&Addr, AccountUnbondingState> = Map::new("account_unbondings");

/// Sequence number used by TS_BALANCE and the sync process
pub const SEQ_NO: Item<Uint64> = Item::new("seq_no");

/// Timeseries for balance changes
pub const TS_BALANCE: Map<(&TokenKey, u64), BalanceEvent> = Map::new("ts_balance");

/// Timeseries for changes to accounts' stake
pub const TS_STAKE: Map<(&Addr, u64), StakingEvent> = Map::new("ts_stake");

/// Aggregate total balance of each token type currently tracked by the house
pub const BALANCES: Map<&TokenKey, TokenAmount> = Map::new("balances");

/// A cyclic buffer of account addresses to sync during amortization
pub const AMORTIZATION_QUEUE: Deque<Addr> = Deque::new("amortization_queue");
