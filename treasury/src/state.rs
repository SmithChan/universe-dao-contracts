use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// Owner If None set, contract is frozen.
    pub owner: Addr,
    pub treasury_amount: Uint128,
    pub treasury_denom: String,
    pub apy: Uint128,
    pub multiple_1: Uint128,
    pub multiple_2: Uint128
}

pub const CONFIG_KEY: &str = "config";
pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);

// histories will take such field : (owner, (address, height, timestamp, action(add or remove), amount))
pub const HISTORIES_KEY: &str = "histories";
pub const HISTORIES: Map<Addr, Vec<(Addr, u64, u64, bool, Uint128)>> = Map::new(HISTORIES_KEY);
