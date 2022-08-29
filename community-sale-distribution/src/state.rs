use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};
use crate::msg::BuyerRecord;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// Owner If None set, contract is frozen.
    pub owner: Addr,
    pub enabled: bool,
    pub address_count: u64,
    pub verse_address: Addr,
    pub verse_amount: Uint128,
    pub steps: u64,
    pub interval: u64
    
}

pub const CONFIG_KEY: &str = "config";
pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);

// histories will take such field : (owner, (address, height, timestamp, action(add or remove), amount))
pub const BUYERS_KEY: &str = "buyers";
pub const BUYERS: Map<Addr, BuyerRecord> = Map::new(BUYERS_KEY);
