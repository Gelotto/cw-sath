use std::collections::HashMap;

use cosmwasm_std::{Addr, Order};

use crate::{
    error::ContractError,
    responses::DepositsResponse,
    state::{
        models::{DepositTokenAmount, Depositor},
        storage::{DEPOSITOR_TOTALS, DEPOSIT_AGG_TOTALS},
    },
    token::Token,
};

use super::ReadonlyContext;

pub fn query_deposits(ctx: ReadonlyContext) -> Result<DepositsResponse, ContractError> {
    let ReadonlyContext { deps, .. } = ctx;

    // Build a hash map from depositor addr to a vec of their total deposits for
    // each token type they've deposited.
    let mut addr2amounts: HashMap<Addr, Vec<DepositTokenAmount>> = HashMap::with_capacity(16);
    for result in DEPOSITOR_TOTALS.range(deps.storage, None, None, Order::Ascending) {
        let ((token_key, addr), totals) = result?;
        let token = Token::from_key(&token_key);
        if let Some(amounts_vec) = addr2amounts.get_mut(&addr) {
            amounts_vec.push(DepositTokenAmount {
                amount: totals.amount,
                n: totals.n,
                token,
            });
        }
    }

    return Ok(DepositsResponse {
        // Aggregate grant total deposit amounts across all depositors
        totals: DEPOSIT_AGG_TOTALS
            .range(deps.storage, None, None, Order::Ascending)
            .map(|r| {
                let (k, v) = r.unwrap();
                DepositTokenAmount {
                    token: Token::from_key(&k),
                    amount: v.amount,
                    n: v.n,
                }
            })
            .collect(),
        // Aggregate total deposit per depositor address
        depositors: addr2amounts
            .iter()
            .map(|(address, totals)| Depositor {
                address: address.to_owned(),
                totals: totals.to_owned(),
            })
            .collect(),
    });
}
