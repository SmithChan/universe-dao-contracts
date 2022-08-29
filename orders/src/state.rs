use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr};
use cw_storage_plus::{Item, Map};

use crate::msg::{LimitConfig, SmartConfig, GridConfig};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub enabled: bool
}


pub const CONFIG_KEY: &str = "config";
pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);

pub const LIMIT_ORDERS_COUNT: Map<Addr, (Vec<u64>, u64)> = Map::new("limit_orders_count");
pub const LIMIT_ORDERS: Map<(Addr, u64), LimitConfig> = Map::new("limit_orders");

pub const SMART_ORDERS_COUNT: Map<Addr, (Vec<u64>, u64)> = Map::new("smart_orders_count");
pub const SMART_ORDERS: Map<(Addr, u64), SmartConfig> = Map::new("smart_orders");

pub const GRID_ORDERS_COUNT: Map<Addr, (Vec<u64>, u64)> = Map::new("grid_orders_count");
pub const GRID_ORDERS: Map<(Addr, u64), GridConfig> = Map::new("grid_orders");

