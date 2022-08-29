use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw20::{Cw20ReceiveMsg};
use cosmwasm_std::{Uint128, Addr};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    pub treasury_address: Addr,
    pub verse_address: Addr,
    pub interval: u64
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakerInfo {
    pub address: Addr,
    pub arr: Vec<StakerRecord>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakerInput {
    pub address: Addr,
    pub amount: Uint128,
    pub apy_type: u64
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StakerRecord {
    pub amount: Uint128,
    pub timestamp: u64,
    pub apy_type: u64
}



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ApyInfo {
    pub timestamp: u64,
    pub apys: Vec<Uint128>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct UnstakingInfo {
    pub amount: Uint128,
    pub timestamp: u64,
    pub apy_type: u64
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HistoryInfo {
    pub amount: Uint128,
    pub timestamp: u64,
    pub is_staking: bool,
    pub apy_type: u64
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateConfig {
        /// NewOwner if non sent, contract gets locked. Recipients can receive airdrops
        /// but owner cannot register new stages.
        new_owner: String,
    },
    UpdateConstants {
        verse_address: Addr,
        treasury_address: Addr,
        sale_address: Addr,
        lock_days: Vec<u64>,
        interval: u64
    },
    UpdateEnabled {
        enabled: bool
    },
    UpdateFetchFromTreasury {
        fetch_from_treasury: bool
    },
    Rebase {
        addresses: Vec<Addr>
        
    },
    Receive(Cw20ReceiveMsg),
    CreateUnstake {
        unstake_amount: Uint128,
        apy_type: u64
    },
    FetchUnstake {
        apy_type: u64,
        index: u64
    },
    AddStakers {
        stakers: Vec<StakerInput>
    },
    RemoveStaker {
        address: Addr,
        apy_type: u64
    },
    RemoveAllStakers {
    },
    SendVerse {
        address: Addr,
        amount: Uint128
    },    
    UpdateApy {
        apy: Uint128,
        multiple_1: Uint128,
        multiple_2: Uint128
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiveMsg {
    Stake{
        apy_type: u64
    },
    Fund {}
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Staker {
        address: Addr
    },
    ListStakers {
        start_after: Option<String>,
        limit: Option<u32>
    },
    Unstaking {
        address: Addr
    },
    Apys {},
    History {
        address: Addr
    }
    
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct ConfigResponse {
    pub owner: Addr,
    pub verse_address: Addr,
    pub treasury_address: Addr,
    pub sale_address: Addr,
    pub stake_amount: Vec<Uint128>,
    pub lock_days: Vec<u64>,
    pub enabled: bool,
    pub last_apy_timestamp: u64,
    pub balance: Uint128,
    pub fetch_from_treasury: bool,
    pub apy: Uint128,
    pub multiple_1: Uint128,
    pub multiple_2: Uint128
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}


#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct StakerListResponse {
    pub stakers: Vec<StakerInfo>,
}


#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct UnstakingResponse {
    pub unstaking: Vec<UnstakingInfo>,
}

/// Returns the vote (opinion as well as weight counted) as well as
/// the address of the voter who submitted it


#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct HistoryResponse {
    pub history: Vec<HistoryInfo>
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct CountInfo {
    pub count: u128
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct TreasuryConfigResponse {
    pub owner: Addr,
    pub treasury_amount: Uint128,
    pub treasury_denom: String,
    pub apy: Uint128,
    pub multiple_1: Uint128,
    pub multiple_2: Uint128
}