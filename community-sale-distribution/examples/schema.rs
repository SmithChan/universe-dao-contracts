use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use universe_community_sale_distribution::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, ReceiveMsg, BuyerRecord, BuyerInput, BuyersResponse, BuyerResponse};

fn main() {
  let mut out_dir = current_dir().unwrap();
  out_dir.push("schema");
  create_dir_all(&out_dir).unwrap();
  remove_schemas(&out_dir).unwrap();

  export_schema(&schema_for!(InstantiateMsg), &out_dir);
  export_schema(&schema_for!(ExecuteMsg), &out_dir);
  export_schema(&schema_for!(QueryMsg), &out_dir);
  export_schema(&schema_for!(ReceiveMsg), &out_dir);
  export_schema(&schema_for!(BuyerRecord), &out_dir);
  export_schema(&schema_for!(BuyerInput), &out_dir);
  export_schema(&schema_for!(BuyersResponse), &out_dir);
  export_schema(&schema_for!(BuyerResponse), &out_dir);
}
