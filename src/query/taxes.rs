use std::collections::HashMap;

use cosmwasm_std::{Addr, Order, StdResult, Uint128};

use crate::{
    error::ContractError,
    math::add_u128,
    responses::{TaxRecipientResponseItem, TaxesResponse},
    state::storage::{TAX_RECIPIENT_CONFIGS, TAX_RECIPIENT_INFOS, TAX_RECIPIENT_TOTALS},
};

use super::ReadonlyContext;

pub fn query_taxes(ctx: ReadonlyContext) -> Result<TaxesResponse, ContractError> {
    let ReadonlyContext { deps, .. } = ctx;

    // Build vec of returned tax recipients
    let mut addr2recipients: HashMap<Addr, TaxRecipientResponseItem> = HashMap::with_capacity(2);
    let mut agg_pct = Uint128::zero();

    for result in TAX_RECIPIENT_INFOS
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<Vec<StdResult<_>>>()
    {
        let (addr, info) = result?;
        let config = TAX_RECIPIENT_CONFIGS.load(deps.storage, &addr)?;

        // Increment running tax pct total
        agg_pct = add_u128(agg_pct, config.pct)?;

        // Create new TaxRecipientResponseItem to return
        if !addr2recipients.contains_key(&addr) {
            addr2recipients.insert(
                addr.to_owned(),
                TaxRecipientResponseItem {
                    address: addr.to_owned(),
                    info,
                    config,
                    totals: vec![],
                },
            );
        }

        let recipient = addr2recipients.get_mut(&addr).unwrap();

        // Build the recipients list of received tax amounts and/or balances
        for result in
            TAX_RECIPIENT_TOTALS
                .prefix(&addr)
                .range(deps.storage, None, None, Order::Ascending)
        {
            let (_, v) = result?;
            recipient.totals.push(v);
        }
    }

    return Ok(TaxesResponse {
        recipients: addr2recipients.values().map(|x| x.to_owned()).collect(),
        pct: agg_pct,
    });
}
