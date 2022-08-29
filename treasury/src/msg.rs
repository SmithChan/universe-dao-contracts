use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Uint128, Addr};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HistoryInfo {
    pub address: Addr, 
    pub height: u64,
    pub timestamp: u64,
    pub is_add: bool,
    pub amount: Uint128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfig {
        new_owner: String,
    },
    UpdateConstants {
        treasury_denom: String
    },
    AddFund {
    },
    RemoveFund {
        amount: Uint128
    },
    RemoveAll {
    },
    UpdateApy {
        apy: Uint128,
        multiple_1: Uint128,
        multiple_2: Uint128
    }
    
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    History {},
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct ConfigResponse {
    pub owner: Addr,
    pub treasury_amount: Uint128,
    pub treasury_denom: String,
    pub apy: Uint128,
    pub multiple_1: Uint128,
    pub multiple_2: Uint128
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}



#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct HistoryResponse {
    pub history: Vec<HistoryInfo>
}


#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct CountInfo {
    pub count: u128
}
