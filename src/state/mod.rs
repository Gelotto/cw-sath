pub mod models;
pub mod storage;

use cosmwasm_std::{Response, Uint128, Uint64};
use models::{TaxRecipientConfig, TaxRecipientInfo};
use storage::{
    CREATED_AT, CREATED_BY, MANAGED_BY, MARKETING_INFO, N_ACCOUNTS, REVENUE_TOKEN_KEYS,
    STAKING_TOKEN, TAX_RECIPIENT_CONFIGS, TAX_RECIPIENT_INFOS, TOTAL_UNBONDING, UNBONDING_SECONDS,
    X,
};

use crate::{error::ContractError, execute::Context, math::add_u128, msg::InstantiateMsg};

use self::storage::{SEQ_NO, TOTAL_DELEGATION};

/// Top-level initialization of contract state
pub fn init(
    ctx: Context,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let Context { deps, info, env } = ctx;
    TOTAL_DELEGATION.save(deps.storage, &Uint128::zero())?;
    TOTAL_UNBONDING.save(deps.storage, &Uint128::zero())?;
    SEQ_NO.save(deps.storage, &Uint64::zero())?;
    X.save(deps.storage, &Uint64::zero())?;
    CREATED_AT.save(deps.storage, &env.block.time)?;
    CREATED_BY.save(deps.storage, &info.sender)?;
    MANAGED_BY.save(deps.storage, &info.sender)?;
    N_ACCOUNTS.save(deps.storage, &0)?;
    MARKETING_INFO.save(deps.storage, &msg.marketing)?;
    STAKING_TOKEN.save(deps.storage, &msg.staking.staking_token)?;
    UNBONDING_SECONDS.save(
        deps.storage,
        &msg.staking.unbonding_seconds.unwrap_or_default(),
    )?;

    for token in msg.staking.revenue_tokens.iter() {
        REVENUE_TOKEN_KEYS.save(deps.storage, &token.to_key(), &0)?;
    }

    // Init taxes
    let mut total_tax_pct = Uint128::zero();
    for info in msg.taxes.iter() {
        let key = deps.api.addr_validate(info.address.as_str())?;

        total_tax_pct = add_u128(total_tax_pct, info.pct)?.min(1_000_000u128.into());
        if total_tax_pct > Uint128::from(1_000_000u128) {
            return Err(ContractError::ValidationError {
                reason: "aggregate tax rate cannot exceed 1000000 or 100%".to_owned(),
            });
        }

        TAX_RECIPIENT_INFOS.save(
            deps.storage,
            &key,
            &TaxRecipientInfo {
                name: info.name.to_owned(),
                logo: info.logo.to_owned(),
            },
        )?;
        TAX_RECIPIENT_CONFIGS.save(
            deps.storage,
            &key,
            &TaxRecipientConfig {
                pct: info.pct,
                autosend: info.autosend,
                immutable: info.immutable,
            },
        )?;
    }

    Ok(Response::new().add_attribute("action", "instantiate"))
}
