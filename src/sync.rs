use std::marker::PhantomData;

use cosmwasm_std::{Addr, Api, Order, Storage, Uint128, Uint64};
use cw_storage_plus::Bound;

use crate::{
    error::ContractError,
    math::{add_u128, mul_ratio_u128, mul_u32, sub_u32},
    state::{
        models::{Account, AccountSyncState, BalanceEvent, StakingEvent},
        storage::{
            ACCOUNTS, ACCOUNT_SYNC_INFOS, AMORTIZATION_QUEUE, N_ACCOUNTS, N_BALANCE_EVENTS,
            TS_BALANCE, TS_STAKE,
        },
    },
    token::Token,
};

pub struct TokenSyncResult {
    pub token: Token,
    pub updated_balance_events: Vec<((String, u64), BalanceEvent)>,
    pub zombie_balance_event_keys: Vec<(String, u64)>,
    pub zombie_delegation_event_keys: Vec<u64>,
    pub synced_amount: Uint128,
}

pub fn amortize(
    store: &mut dyn Storage,
    api: &dyn Api,
    seq_no: Uint64,
    token: Option<Token>,
    ignore_address: Option<Addr>,
) -> Result<(), ContractError> {
    let n_accounts = N_ACCOUNTS.load(store)?;
    let queue_size = AMORTIZATION_QUEUE.len(store)?;

    if queue_size > 0 {
        let n = 10
            .min(queue_size)
            .min((mul_u32(n_accounts, 50_000)?) / 1_000_000);

        for _ in 0..n {
            if let Some(addr) = AMORTIZATION_QUEUE.pop_front(store)? {
                if let Some(ignore_addr) = &ignore_address {
                    if addr == *ignore_addr {
                        AMORTIZATION_QUEUE.push_back(store, &addr)?;
                        continue;
                    }
                }
                if let Some(account) = ACCOUNTS.may_load(store, &addr)? {
                    let results =
                        sync_account(store, api, &addr, &account, seq_no, token.to_owned(), false)?;
                    for (result, state) in results.iter() {
                        persist_sync_results(store, &addr, result, state)?;
                    }
                    AMORTIZATION_QUEUE.push_back(store, &addr)?;
                }
            }
        }
    }
    Ok(())
}

pub fn persist_sync_results(
    store: &mut dyn Storage,
    staker: &Addr,
    result: &TokenSyncResult,
    sync_state: &AccountSyncState,
) -> Result<(), ContractError> {
    ACCOUNT_SYNC_INFOS.save(store, (&staker, &result.token.to_key()), sync_state)?;

    for ((a, b), v) in result.updated_balance_events.iter() {
        TS_BALANCE.save(store, (a, *b), v)?;
    }

    N_BALANCE_EVENTS.update(
        store,
        &result.token.to_key(),
        |maybe_n| -> Result<_, ContractError> {
            sub_u32(
                maybe_n.unwrap_or_default(),
                result.zombie_balance_event_keys.len() as u32,
            )
        },
    )?;

    for key in result.zombie_balance_event_keys.iter() {
        let (token_key, seq_no) = key;
        TS_BALANCE.remove(store, (token_key, *seq_no));
    }

    for seq_no in result.zombie_delegation_event_keys.iter() {
        TS_STAKE.remove(store, (staker, *seq_no));
    }

    Ok(())
}

pub fn sync_account(
    store: &dyn Storage,
    api: &dyn Api,
    address: &Addr,
    account: &Account,
    seq_no: Uint64,
    token: Option<Token>,
    terminal: bool,
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
        if let Some(result) = sync_account_balance(
            store,
            api,
            &address,
            &mut sync_state,
            &token,
            seq_no,
            terminal,
        )? {
            retval.push((result, sync_state));
        }
    } else {
        for result in ACCOUNT_SYNC_INFOS
            .prefix(address)
            .range(store, None, None, Order::Ascending)
        {
            let (token_key, mut sync_state) = result?;
            let token = Token::from_key(&token_key);
            if let Some(result) = sync_account_balance(
                store,
                api,
                &address,
                &mut sync_state,
                &token,
                seq_no,
                terminal,
            )? {
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
    seq_no: Uint64,
    terminal: bool,
) -> Result<Option<TokenSyncResult>, ContractError> {
    // Compute delegator account's aggregate synced revenue amount
    if let Some(result) = perform_sync(store, api, delegator, sync, token, seq_no, terminal)? {
        // Update the sync state of the delegator's account
        sync.amount = add_u128(sync.amount, result.synced_amount)?;
        sync.seq_no = seq_no;
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
    seq_no: Uint64,
    _terminal: bool,
) -> Result<Option<TokenSyncResult>, ContractError> {
    let mut delegation_events = load_delegation_events(store, &delegator, sync_state, seq_no)?;
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
        delegation_events.push((seq_no.u64() + 1u64, dummy_event));
    }

    let token_key = token.to_key();

    // Return these keys and delete the events in caller
    let mut updated_balance_events: Vec<((String, u64), BalanceEvent)> = Vec::with_capacity(2);
    let mut zombie_balance_event_keys: Vec<(String, u64)> = Vec::with_capacity(2);
    let mut zombie_delegation_event_keys: Vec<u64> = Vec::with_capacity(2);

    // Iterate through pairs of delegation events, accumulating the
    // aggregate sync amount within the range between by each pair.
    for (i, (s1, e1)) in delegation_events
        .iter()
        .take(delegation_events.len() - 1)
        .enumerate()
    {
        let account_deleg = e1.delta;
        let (s2, _) = &delegation_events[i + 1];

        api.debug(format!("processing delegation events btw: {:?} - {:?}", s1, s2).as_str());

        // Accumulate the delegator's share of revenue between the given
        // delegation events.
        for result in TS_BALANCE.range(
            store,
            Some(Bound::Inclusive(((&token_key, *s1), PhantomData))),
            Some(Bound::Exclusive(((&token_key, *s2), PhantomData))),
            Order::Ascending,
        ) {
            let (k, mut e) = result?;

            api.debug(format!("processing balance event delta: {:?}", e.delta).as_str());

            zombie_delegation_event_keys.push(*s1);

            // Compute the delegator's share of revenue for this
            // RevenueEvent based on their delegation compared to total
            // delegation across all accounts at that time, and increment
            // the running total sync amount.
            let account_revenue = mul_ratio_u128(e.delta, account_deleg, e.total)?;
            agg_sync_amount = add_u128(agg_sync_amount, account_revenue)?;

            e.ref_count = sub_u32(e.ref_count, 1)?;

            if e.ref_count == 0 {
                zombie_balance_event_keys.push(k);
            } else {
                updated_balance_events.push((k, e));
            }
        }
    }

    Ok(Some(TokenSyncResult {
        updated_balance_events,
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
    seq_no: Uint64,
) -> Result<Vec<(u64, StakingEvent)>, ContractError> {
    let mut events: Vec<(u64, StakingEvent)> = Vec::with_capacity(8);
    for result in TS_STAKE.range(
        store,
        Some(Bound::Inclusive((
            (&delegator, sync.seq_no.u64()),
            PhantomData,
        ))),
        Some(Bound::Exclusive(((&delegator, seq_no.u64()), PhantomData))),
        Order::Ascending,
    ) {
        let ((_, seq_no), event) = result?;
        events.push((seq_no, event));
    }
    Ok(events)
}
