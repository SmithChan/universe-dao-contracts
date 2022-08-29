use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use cosmwasm_std::Coin;

use universe_staking::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, StakerInfo, StakerInput, StakerRecord, ApyInfo, UnstakingInfo, HistoryInfo, 
ConfigResponse, StakerListResponse, UnstakingResponse, HistoryResponse, CountInfo, TreasuryConfigResponse};

fn main() {
  let mut out_dir = current_dir().unwrap();
  out_dir.push("schema");
  create_dir_all(&out_dir).unwrap();
  remove_schemas(&out_dir).unwrap();

  export_schema(&schema_for!(InstantiateMsg), &out_dir);
  export_schema(&schema_for!(ExecuteMsg), &out_dir);
  export_schema(&schema_for!(QueryMsg), &out_dir);
  export_schema(&schema_for!(StakerInfo), &out_dir);
  export_schema(&schema_for!(StakerInput), &out_dir);
  export_schema(&schema_for!(StakerRecord), &out_dir);
  export_schema(&schema_for!(ApyInfo), &out_dir);
  export_schema(&schema_for!(UnstakingInfo), &out_dir);
  export_schema(&schema_for!(HistoryInfo), &out_dir);
  export_schema(&schema_for!(ConfigResponse), &out_dir);
  export_schema(&schema_for!(StakerListResponse), &out_dir);
  export_schema(&schema_for!(UnstakingResponse), &out_dir);
  export_schema(&schema_for!(HistoryResponse), &out_dir);
  export_schema(&schema_for!(CountInfo), &out_dir);
  export_schema(&schema_for!(TreasuryConfigResponse), &out_dir);
  
}
