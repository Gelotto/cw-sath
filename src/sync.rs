use std::marker::PhantomData;

use cosmwasm_std::{Addr, Api, Order, Storage, Timestamp, Uint128, Uint64};
use cw_storage_plus::{Bound, PrefixBound};

use crate::{
    error::ContractError,
    math::{add_u128, mul_ratio_u128},
    state::{
        models::{Account, DelegationEvent, RevenueEvent},
        storage::{DELEGATION_EVENTS, REVENUE_EVENTS},
    },
};
pub fn sync_account(
    store: &dyn Storage,
    api: &dyn Api,
    account: &mut Account,
    delegator: &Addr,
    now: Timestamp,
    seq_no: Uint64,
) -> Result<(), ContractError> {
    // Compute delegator account's aggregate synced revenue amount
    let sync_amount = perform_sync(store, api, delegator, account, now, seq_no)?;

    // Update the sync state of the delegator's account
    account.sync.amount = add_u128(account.sync.amount, sync_amount)?;
    account.sync.seq_no = seq_no;
    account.sync.t = now;

    Ok(())
}

fn perform_sync(
    store: &dyn Storage,
    api: &dyn Api,
    delegator: &Addr,
    account: &Account,
    now: Timestamp,
    seq_no: Uint64,
) -> Result<Uint128, ContractError> {
    let mut delegation_events = load_delegation_events(store, &delegator, account, now)?;
    let mut agg_sync_amount = Uint128::zero();

    api.debug(format!("delegation events: {:?}", delegation_events).as_str());

    if delegation_events.is_empty() {
        return Ok(Uint128::zero());
    }
    // The following for-loop iterates through successive pairs of
    // delegation_events, so if there's only one, we need to add a dummy
    // event so that there is at least one pair.
    if delegation_events.len() == 1 {
        let dummy_event = DelegationEvent::default();
        delegation_events.push((seq_no.u64(), now.nanos(), dummy_event))
    }

    // Iterate through pairs of delegation events, accumulating the
    // aggregate sync amount within the range between by each pair.
    for (i, (s1, t1, e1)) in delegation_events
        .iter()
        .take(delegation_events.len() - 1)
        .enumerate()
    {
        let account_deleg = e1.d;
        let (s2, t2, _) = &delegation_events[i + 1];

        // Accumulate the delegator's share of revenue between the given
        // delegation events.
        for result in REVENUE_EVENTS.range(
            store,
            Some(Bound::Inclusive(((*t1, *s1), PhantomData))),
            Some(Bound::Exclusive(((*t2, *s2), PhantomData))),
            Order::Ascending,
        ) {
            let (
                _,
                RevenueEvent {
                    r: revenue,
                    d: total_deleg,
                },
            ) = result?;

            // Compute the delegator's share of revenue for this
            // RevenueEvent based on their delegation compared to total
            // delegation across all accounts at that time, and increment
            // the running total sync amount.
            let account_revenue = mul_ratio_u128(revenue, account_deleg, total_deleg)?;
            agg_sync_amount = add_u128(agg_sync_amount, account_revenue)?;

            api.debug(format!("event revenue: {:?}", revenue.u128()).as_str());
            api.debug(format!("account delegation: {:?}", account_deleg.u128()).as_str());
            api.debug(format!("contract delegation: {:?}", total_deleg.u128()).as_str());
            api.debug(format!("account revenue: {:?}", account_revenue.u128()).as_str());
        }
    }

    // TODO: synchronously remove all stale events from storage
    Ok(agg_sync_amount)
}

pub fn load_delegation_events(
    store: &dyn Storage,
    delegator: &Addr,
    account: &Account,
    now: Timestamp,
) -> Result<Vec<(u64, u64, DelegationEvent)>, ContractError> {
    let mut events: Vec<(u64, u64, DelegationEvent)> = Vec::with_capacity(8);
    for result in DELEGATION_EVENTS.prefix_range(
        store,
        Some(PrefixBound::Inclusive((
            (&delegator, account.sync.t.nanos()),
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
