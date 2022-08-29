use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// Owner If None set, contract is frozen.
    pub owner: Addr,
    pub verse_address: Addr,
    pub treasury_address: Addr,
    pub sale_address: Addr,
    pub stake_amount: Vec<Uint128>,
    pub lock_days: Vec<u64>,
    pub enabled: bool,
    pub last_apy_timestamp: u64,
    pub balance: Uint128,
    pub interval: u64,

    pub fetch_from_treasury: bool,
    pub apy: Uint128,
    pub multiple_1: Uint128,
    pub multiple_2: Uint128
}

pub const CONFIG_KEY: &str = "config";
pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);

// STAKER : <address, (amount, timestamp, apy_type)>
pub const STAKERS_KEY: &str = "stakers";
pub const STAKERS: Map<Addr, Vec<(Uint128, u64, u64)>> = Map::new(STAKERS_KEY);

// UNSTAKING: <address, Vec<(amount, timestamp, apy_type)>>
pub const UNSTAKING_KEY: &str = "unstaking";
pub const UNSTAKING: Map<Addr, Vec<(Uint128, u64, u64)>> = Map::new(UNSTAKING_KEY);

// HISTORIES : <address, Vec<(amount, timestamp, action, apy_type)>>
pub const HISTORIES_KEY: &str = "histories";
pub const HISTORIES: Map<Addr, Vec<(Uint128, u64, bool, u64)>> = Map::new(HISTORIES_KEY);

//APYS: <timestamp, Vec<apy>>
pub const APYS_KEY: &str = "apys";
pub const APYS: Map<u64, Vec<Uint128>> = Map::new(APYS_KEY);