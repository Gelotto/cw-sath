pub mod account;
pub mod house;

use cosmwasm_std::{Deps, Env};

pub struct ReadonlyContext<'a> {
    pub deps: Deps<'a>,
    pub env: Env,
}
