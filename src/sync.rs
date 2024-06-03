use std::marker::PhantomData;

use cosmwasm_std::{Addr, Api, Order, Storage, Timestamp, Uint128, Uint64};
use cw_storage_plus::{Bound, PrefixBound};

use crate::{
    error::ContractError,
    math::{add_u128, mul_ratio_u128},
    state::{
        models::{Account, AccountSyncState, BalanceEvent, StakingEvent},
        storage::{
            ACCOUNT_SYNC_INFOS, BALANCE_EVENTS, CONFIG_LIQUIDITY_TOKENS, QUEUE, STAKING_EVENTS,
        },
    },
    token::Token,
};

pub struct TokenSyncResult {
    pub token: Token,
    pub zombie_balance_event_keys: Vec<(String, u64, u64)>,
    pub zombie_delegation_event_keys: Vec<(u64, u64)>,
    pub synced_amount: Uint128,
}

pub fn amortize(
    store: &mut dyn Storage,
    time: Timestamp,
    seq_no: Uint64,
    n: u32,
    token: Option<Token>,
    ignore_address: Option<Addr>,
) -> Result<(), ContractError> {
    // let queue_size = QUEUE.len(store)?;
    // if queue_size > 0 {
    //     let n = n.min(queue_size);
    //     for _ in 0..n {
    //         if let Some(addr) = QUEUE.pop_front(store)? {
    //             if let Some(ignore_addr) = &ignore_address {
    //                 if addr == *ignore_addr {
    //                     QUEUE.push_back(store, &addr)?;
    //                     continue;
    //                 }
    //             }
    //             let results = sync_account(store, &addr, time, seq_no, token.to_owned())?;
    //             for (result, state) in results.iter() {
    //                 persist_sync_results(store, &addr, result, state)?;
    //             }

    //             QUEUE.push_back(store, &addr)?;
    //         }
    //     }
    // }
    Ok(())
}

pub fn persist_sync_results(
    store: &mut dyn Storage,
    staker: &Addr,
    result: &TokenSyncResult,
    sync_state: &AccountSyncState,
) -> Result<(), ContractError> {
    ACCOUNT_SYNC_INFOS.save(store, (&staker, &result.token.to_key()), sync_state)?;

    for key in result.zombie_balance_event_keys.iter() {
        let (a, b, c) = key;
        BALANCE_EVENTS.remove(store, (a, *b, *c));
    }

    for key in result.zombie_delegation_event_keys.iter() {
        let (b, c) = key;
        STAKING_EVENTS.remove(store, (staker, *b, *c));
    }

    Ok(())
}

pub fn sync_account(
    store: &dyn Storage,
    api: &dyn Api,
    address: &Addr,
    account: &Account,
    t: Timestamp,
    seq_no: Uint64,
    token: Option<Token>,
) -> Result<Vec<(TokenSyncResult, AccountSyncState)>, ContractError> {
    let mut retval: Vec<(TokenSyncResult, AccountSyncState)> = Vec::with_capacity(2);
    if let Some(token) = token {
        let mut sync_state = ACCOUNT_SYNC_INFOS
            .may_load(store, (address, &token.to_key()))?
            .or_else(|| {
                Some(AccountSyncState {
                    amount: Uint128::zero(),
                    seq_no: account.created_at_seq_no,
                    t: account.created_at,
                })
            })
            .unwrap();
        if let Some(result) =
            sync_account_balance(store, api, &address, &mut sync_state, &token, t, seq_no)?
        {
            retval.push((result, sync_state));
        }
    } else {
        for result in ACCOUNT_SYNC_INFOS
            .prefix(address)
            .range(store, None, None, Order::Ascending)
        {
            let (token_key, mut sync_state) = result?;
            let token = CONFIG_LIQUIDITY_TOKENS.load(store, &token_key)?;
            if let Some(result) =
                sync_account_balance(store, api, &address, &mut sync_state, &token, t, seq_no)?
            {
                retval.push((result, sync_state));
            }
        }
    }
    Ok(retval)
}

pub fn sync_account_balance(
    store: &dyn Storage,
    api: &dyn Api,
    delegator: &Addr,
    sync: &mut AccountSyncState,
    token: &Token,
    now: Timestamp,
    seq_no: Uint64,
) -> Result<Option<TokenSyncResult>, ContractError> {
    // Compute delegator account's aggregate synced revenue amount
    if let Some(result) = perform_sync(store, api, delegator, sync, token, now, seq_no)? {
        // Update the sync state of the delegator's account
        sync.amount = add_u128(sync.amount, result.synced_amount)?;
        sync.seq_no = seq_no;
        sync.t = now;
        return Ok(Some(result));
    }

    Ok(None)
}

fn perform_sync(
    store: &dyn Storage,
    api: &dyn Api,
    delegator: &Addr,
    sync_state: &AccountSyncState,
    token: &Token,
    now: Timestamp,
    seq_no: Uint64,
) -> Result<Option<TokenSyncResult>, ContractError> {
    let mut delegation_events = load_delegation_events(store, &delegator, sync_state, now)?;
    let mut agg_sync_amount = Uint128::zero();

    api.debug(format!("token: {:?}", token).as_str());
    api.debug(format!("seq_no: {:?}", seq_no).as_str());
    api.debug(format!("initial sync state: {:?}", sync_state).as_str());
    api.debug(format!("delegation events: {:?}", delegation_events).as_str());

    if delegation_events.is_empty() {
        return Ok(None);
    }
    // The following for-loop iterates through successive pairs of
    // delegation_events, so if there's only one, we need to add a dummy
    // event so that there is at least one pair.
    if delegation_events.len() == 1 {
        let dummy_event = StakingEvent::default();
        delegation_events.push((seq_no.u64(), now.nanos(), dummy_event))
    }

    let token_key = token.to_key();

    // TODO: return these keys and delete the events in caller
    let mut zombie_balance_event_keys: Vec<(String, u64, u64)> = Vec::with_capacity(2);
    let mut zombie_delegation_event_keys: Vec<(u64, u64)> = Vec::with_capacity(2);

    // Iterate through pairs of delegation events, accumulating the
    // aggregate sync amount within the range between by each pair.
    for (i, (s1, t1, e1)) in delegation_events
        .iter()
        .take(delegation_events.len() - 1)
        .enumerate()
    {
        let account_deleg = e1.delta;
        let (s2, t2, _) = &delegation_events[i + 1];

        // Accumulate the delegator's share of revenue between the given
        // delegation events.
        for result in BALANCE_EVENTS.range(
            store,
            Some(Bound::Inclusive(((&token_key, *t1, *s1), PhantomData))),
            Some(Bound::Exclusive(((&token_key, *t2, *s2), PhantomData))),
            Order::Ascending,
        ) {
            let (
                _,
                BalanceEvent {
                    delta: revenue,
                    total: total_deleg,
                    ref_count,
                    ..
                },
            ) = result?;

            if ref_count - 1 == 0 {
                zombie_balance_event_keys.push((token_key.to_owned(), *t1, *t2));
            }

            zombie_delegation_event_keys.push((*t1, *s1));

            // Compute the delegator's share of revenue for this
            // RevenueEvent based on their delegation compared to total
            // delegation across all accounts at that time, and increment
            // the running total sync amount.
            let account_revenue = mul_ratio_u128(revenue, account_deleg, total_deleg)?;
            agg_sync_amount = add_u128(agg_sync_amount, account_revenue)?;
        }
    }

    // TODO: synchronously remove all stale events from storage
    Ok(Some(TokenSyncResult {
        zombie_balance_event_keys,
        zombie_delegation_event_keys,
        synced_amount: agg_sync_amount,
        token: token.to_owned(),
    }))
}

pub fn load_delegation_events(
    store: &dyn Storage,
    delegator: &Addr,
    sync: &AccountSyncState,
    now: Timestamp,
) -> Result<Vec<(u64, u64, StakingEvent)>, ContractError> {
    let mut events: Vec<(u64, u64, StakingEvent)> = Vec::with_capacity(8);
    for result in STAKING_EVENTS.prefix_range(
        store,
        Some(PrefixBound::Inclusive((
            (&delegator, sync.t.nanos()),
            PhantomData,
        ))),
        Some(PrefixBound::Exclusive((
            (&delegator, now.nanos()),
            PhantomData,
        ))),
        Order::Ascending,
    ) {
        let ((_, t, seq_no), event) = result?;
        events.push((seq_no, t, event));
    }
    Ok(events)
}
