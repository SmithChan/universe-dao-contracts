#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, to_binary, from_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, CosmosMsg, WasmMsg,
     Addr, Order
};
use cw_storage_plus::Bound;
use cw_utils::{maybe_addr};
use cw20::{Cw20ReceiveMsg, Cw20ExecuteMsg};
use cw2::{get_contract_version, set_contract_version};
use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, ReceiveMsg, BuyerRecord, BuyerInput, BuyersResponse, BuyerResponse
};
use crate::state::{
    Config, CONFIG, BUYERS
};

// Version info, for migration info
const CONTRACT_NAME: &str = "universe_community_sale_distribution";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        owner: info.sender.clone(),
        enabled: false,
        address_count: 0u64,
        verse_address: msg.verse_address,
        verse_amount: Uint128::zero(),
        steps: msg.steps,
        interval: msg.interval
    };
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig { owner } => execute_update_config(deps, info, owner),
        ExecuteMsg::UpdateEnabled { enabled } => execute_update_enabled(deps, info, enabled),
        ExecuteMsg::Receive( msg ) => execute_receive(deps, info, msg),
        ExecuteMsg::AddBuyers { list } => execute_add_buyers(deps, env, info, list),
        ExecuteMsg::Claim{ } => execute_claim(deps, env, info)
    }
}

pub fn check_owner(
    deps: &DepsMut,
    info: &MessageInfo
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    if info.sender != cfg.owner {
        return Err(ContractError::Unauthorized {})
    }
    Ok(Response::new().add_attribute("action", "check_owner"))
}

pub fn check_enabled(
    deps: &DepsMut,
    _info: &MessageInfo
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    if !cfg.enabled {
        return Err(ContractError::Disabled {})
    }
    Ok(Response::new().add_attribute("action", "check_enabled"))
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Addr
) -> Result<Response, ContractError> {
    // authorize owner
    check_owner(&deps, &info)?;
    
    CONFIG.update(deps.storage, |mut exists| -> StdResult<_> {
        exists.owner = owner;
        Ok(exists)
    })?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

pub fn execute_update_enabled(
    deps: DepsMut,
    info: MessageInfo,
    enabled: bool
) -> Result<Response, ContractError> {
    // authorize owner
    check_owner(&deps, &info)?;
    
    CONFIG.update(deps.storage, |mut exists| -> StdResult<_> {
        exists.enabled = enabled;
        Ok(exists)
    })?;

    Ok(Response::new().add_attribute("action", "update_enabled"))
}


pub fn execute_receive(
    deps: DepsMut,
    info: MessageInfo,
    wrapper: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {

    let cfg = CONFIG.load(deps.storage)?;
    
    let msg: ReceiveMsg = from_binary(&wrapper.msg)?;
    let user_addr = &deps.api.addr_validate(&wrapper.sender)?;

    if wrapper.amount == Uint128::zero() {
        return Err(ContractError::InvalidInput {});
    }

    if info.sender != cfg.verse_address {
        return Err(ContractError::UnacceptableToken {});
    }
    match msg {
        ReceiveMsg::Fund { } => {
            
            CONFIG.update(deps.storage, |mut exists| -> StdResult<_> {
                exists.verse_amount += wrapper.amount;
                Ok(exists)
            })?;
            return Ok(Response::new()
                .add_attributes(vec![
                    attr("action", "fund"),
                    attr("amount", wrapper.amount),
                    attr("address", user_addr.to_string())
                ]));
        }
    }
}

pub fn execute_add_buyers(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    list: Vec<BuyerInput>
) -> Result<Response, ContractError> {

    check_owner(&deps, &info)?;

    let mut count = 0u64;
    for item in list.clone() {
        if !BUYERS.has(deps.storage, item.address.clone()) {
            count += 1;
        }
        BUYERS.save(deps.storage, item.address.clone(), &BuyerRecord { initial_amount: item.amount, claimed_amount: Uint128::zero(), last_timestamp: 0u64 })?;
    }

    CONFIG.update(deps.storage, |mut exists| -> StdResult<_> {
        exists.address_count += count;
        Ok(exists)
    })?;
    
    return Ok(Response::new()
        .add_attributes(vec![
            attr("action", "add_buyers"),
            attr("count", list.clone().len().to_string()),
        ]));
}

pub fn execute_claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo
) -> Result<Response, ContractError> {

    check_enabled(&deps, &info)?;

    let mut cfg = CONFIG.load(deps.storage)?;
    
    let mut record = BUYERS.load(deps.storage, info.sender.clone())?;
    
    if record.claimed_amount == record.initial_amount {
        return Err(ContractError::AlreadyClaimedAll {});
    }

    let mut amount;
    if record.last_timestamp == 0u64 {
        amount = record.initial_amount / Uint128::from(cfg.steps);
    } else {
        amount = record.initial_amount / Uint128::from(cfg.steps) * Uint128::from(env.block.time.seconds() / cfg.interval - record.last_timestamp / cfg.interval);
        if amount > record.initial_amount - record.claimed_amount {
            amount = record.initial_amount - record.claimed_amount;
        }
    }
    if cfg.verse_amount < amount {
        return Err(ContractError::NotEnoughTokens {});
    }

    record.last_timestamp = env.block.time.seconds();
    record.claimed_amount += amount;
    cfg.verse_amount -= amount;
    CONFIG.save(deps.storage, &cfg)?;
    BUYERS.save(deps.storage, info.sender.clone(), &record)?;
    
    let mut messages:Vec<CosmosMsg> = vec![];
            
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: cfg.verse_address.clone().into(),
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: info.sender.clone().into(),
            amount
        })?,
    }));

    
    return Ok(Response::new()
        .add_messages(messages)
        .add_attributes(vec![
            attr("action", "claim"),
            attr("address", info.sender.clone()),
            attr("amount", amount),
        ]));
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} 
            => to_binary(&query_config(deps)?),
        QueryMsg::Buyers {start_after, limit}
            => to_binary(&query_buyers(deps, start_after, limit)?),
        QueryMsg::Buyer {address}
            => to_binary(&query_buyer(deps, address)?)
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let cfg = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: cfg.owner,
        enabled: cfg.enabled,
        address_count: cfg.address_count,
        verse_address: cfg.verse_address,
        verse_amount: cfg.verse_amount,
        steps: cfg.steps,
        interval: cfg.interval
    })
}


fn map_buyer(
    item: StdResult<(Addr, BuyerRecord)>,
) -> StdResult<BuyerResponse> {
    item.map(|(address, record)| {
        BuyerResponse {
            address,
            initial_amount: record.initial_amount,
            claimed_amount: record.claimed_amount,
            last_timestamp: record.last_timestamp
        }
    })
}

const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

fn query_buyers(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<BuyersResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let addr = maybe_addr(deps.api, start_after)?;
    let start = addr.map(|addr| Bound::exclusive(addr));

    let buyers:StdResult<Vec<_>> = BUYERS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| map_buyer(item))
        .collect();

    Ok(BuyersResponse { buyers: buyers? })
}

fn query_buyer(deps: Deps, address: Addr) -> StdResult<BuyerResponse> {
    let record = BUYERS.load(deps.storage, address.clone())?;
    Ok(BuyerResponse {
        address,
        initial_amount: record.initial_amount,
        claimed_amount: record.claimed_amount,
        last_timestamp: record.last_timestamp
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let version = get_contract_version(deps.storage)?;
    if version.contract != CONTRACT_NAME {
        return Err(ContractError::CannotMigrate {
            previous_contract: version.contract,
        });
    }
    Ok(Response::default())
}

