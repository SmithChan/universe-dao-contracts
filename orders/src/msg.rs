use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw20::{Cw20ReceiveMsg};
use cosmwasm_std::{Uint128, Addr};
use cw20::{Denom};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateOwner {
        owner: Addr,
    },
    UpdateEnabled {
        enabled: bool
    },
    Receive(Cw20ReceiveMsg),

    Stop {
        order_type: u64,
        id: u64
    },
    Sync {
        order_type: u64,
        address: Option<Addr>,
        id: u64
    },

    StartLimit(LimitMsg),
    StartSmart(SmartMsg),
    StartGrid(GridMsg),
    Withdraw {
        denom: Denom
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct OrderAddressesResponse {
    pub addresses: Vec<Addr>,
}
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct OrderForAddressIdsResponse {
    pub address: Addr,
    pub ids: Vec<u64>
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct OrderResponse {
    pub address: Addr,
    pub id: u64,
    pub limit_order: Option<LimitConfig>,
    pub smart_order: Option<SmartConfig>,
    pub grid_order: Option<GridConfig>,
}



#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct OrdersResponse {
    pub address: Addr,
    pub limit_orders: Option<Vec<LimitConfig>>,
    pub smart_orders: Option<Vec<SmartConfig>>,
    pub grid_orders: Option<Vec<GridConfig>>,
}
/// Limit Order ///////////////////////////////////////////////////////////////////////////////////////////
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct LimitMsg {
    pub token1_denom: Denom, // {"cw20":"address"} or {"native":"ujuno"}
    pub pool_address: Addr, // pool address
    pub take_profit_percentage: u64 // minimum advantage rate to sell
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LimitConfig {
    pub msg: LimitMsg,
    pub token2_denom: Denom,
    pub initial_token1_amount: Uint128,
    pub token1_amount: Uint128,
    pub token2_amount: Uint128,
    pub avg_buy_price: Uint128,
    pub target_buy_price: Uint128,
    pub finished: bool
}

/// Smart Order ///////////////////////////////////////////////////////////////////////////////////////////
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SmartMsg {
    pub token1_denom: Denom, // {"cw20":"address"} or {"native":"ujuno"}
    pub pool_address: Addr, // pool address
    pub take_profit_percentage: u64, // percentage above the average_purchase_price at which it should take profit
    pub initial_token1_amount: Uint128, // initial buy amount, not same as input amount
    pub num_dca_orders: u64, // number of orders created for double cost averaging
    pub dca_step: u64, // difference of the price drop between dca_orders
    pub dca_step_multiplier: u64, // multiplier of the dca_step
    pub dca_order_size: Uint128, // order size for each dca_order
    pub dca_order_size_multiplier: u64 // multiplier of the dca_order_size
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SmartConfig {
    pub msg: SmartMsg,
    pub token2_denom: Denom,
    pub token1_amount: Uint128,
    pub token2_amount: Uint128,
    pub avg_buy_price: Uint128,
    pub target_buy_price: Uint128,
    pub finished: bool,
    pub dca_prices: Vec<Uint128>,
    pub dca_amounts: Vec<Uint128>,
    pub current_dca_point: u64
}


/// Grid Order ///////////////////////////////////////////////////////////////////////////////////////////
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct GridMsg {
    pub token1_denom: Denom, // {"cw20":"address"} or {"native":"ujuno"}
    pub pool_address: Addr, // pool address
    pub total_amount: Uint128, // total input amount
    pub num_grid_pairs: u64, // number of orders created for double cost averaging
    pub price_range_percentage: u64, // -10%~10%
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GridConfig {
    pub msg: GridMsg, 
    pub token2_denom: Denom,
    pub buy_prices: Vec<Uint128>, // The case when the second token price goes down
    pub sell_prices: Vec<Uint128>, // The case when the second token price goes up
    pub order_amount: Uint128,
    pub finished: bool,
    pub buy_step: u64,
    pub sell_step: u64,
    pub token1_amount: Uint128,
    pub token2_amount: Uint128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    Limit(LimitMsg),
    Smart(SmartMsg),
    Grid(GridMsg)
    
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},

    OrderAddresses {
        order_type: u64,
        start_after: Option<String>,
        limit: Option<u32>,
    },
    OrderForAddressIds {order_type: u64, address: Addr},
    Order {order_type: u64, address: Addr, id: u64},
    Orders {order_type: u64, address: Addr},
}




#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct ConfigResponse {
    pub owner: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

