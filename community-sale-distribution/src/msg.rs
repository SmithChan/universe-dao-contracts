use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Uint128, Addr};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    pub verse_address: Addr,
    pub steps: u64,
    pub interval: u64
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BuyerInput {
    pub address: Addr, 
    pub amount: Uint128
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct BuyerRecord {
    pub initial_amount: Uint128,
    pub claimed_amount: Uint128,
    pub last_timestamp: u64
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct BuyerResponse {
    pub address: Addr,
    pub initial_amount: Uint128,
    pub claimed_amount: Uint128,
    pub last_timestamp: u64
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfig {
        owner: Addr,
    },
    UpdateEnabled {
        enabled: bool
    },
    AddBuyers {
        list: Vec<BuyerInput>
    },
    Claim {
    },
    Receive(Cw20ReceiveMsg)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    Fund{}
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Buyers {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    Buyer {
        address: Addr
    }
}
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct ConfigResponse {
    pub owner: Addr,
    pub enabled: bool,
    pub address_count: u64,
    pub verse_address: Addr,
    pub verse_amount: Uint128,
    pub steps: u64,
    pub interval: u64
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct BuyersResponse {
    pub buyers: Vec<BuyerResponse>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

