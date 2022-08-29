#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, from_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr, Order
};
use cw_storage_plus::Bound;
use cw_utils::maybe_addr;
use cw2::{get_contract_version, set_contract_version};
use cw20::{Balance, Cw20CoinVerified, Cw20ReceiveMsg, Denom};
use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, ReceiveMsg, OrderAddressesResponse, OrderForAddressIdsResponse, OrderResponse, OrdersResponse, LimitConfig, SmartConfig, GridConfig
};
use crate::state::{
    Config, CONFIG, LIMIT_ORDERS, LIMIT_ORDERS_COUNT, SMART_ORDERS, SMART_ORDERS_COUNT, GRID_ORDERS, GRID_ORDERS_COUNT
};

use crate::ordergroup;
use crate::util;

// Version info, for migration info
const CONTRACT_NAME: &str = "universe_orders";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        owner: info.sender.clone(),
        enabled: true
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
        ExecuteMsg::UpdateOwner { owner } => util::execute_update_owner(deps.storage, info.sender, owner),
        ExecuteMsg::UpdateEnabled { enabled } => util::execute_update_enabled(deps.storage, info.sender, enabled),
        ExecuteMsg::Receive( msg ) => execute_receive(deps, info, msg),

        ExecuteMsg::Stop { order_type, id } => {
            if order_type == 0u64 {
                ordergroup::stop_limit(deps, info.sender, id)
            } else if order_type == 1u64 {
                ordergroup::stop_smart(deps, info.sender, id)
            } else if order_type == 2u64 {
                ordergroup::stop_grid(deps, info.sender, id)
            } else {
                ordergroup::stop_grid(deps, info.sender, id)
            }
        },
        ExecuteMsg::Sync { order_type, address, id } => {
            if order_type == 0u64 {
                ordergroup::sync_limit(deps, info.sender, address, id, false)
            } else if order_type == 1u64 {
                ordergroup::sync_smart(deps, info.sender, address, id, false)
            } else if order_type == 2u64 {
                ordergroup::sync_grid(deps, info.sender, address, id, false)
            } else {
                ordergroup::sync_grid(deps, info.sender, address, id, false)
            }
        },
        ExecuteMsg::StartLimit( msg ) => ordergroup::start_limit(deps, msg, Balance::from(info.funds), info.sender),
        ExecuteMsg::StartSmart( msg ) => ordergroup::start_smart(deps, msg, Balance::from(info.funds), info.sender),
        ExecuteMsg::StartGrid( msg ) => ordergroup::start_grid(deps, msg, Balance::from(info.funds), info.sender),

        ExecuteMsg::Withdraw { denom } => execute_withdraw(deps, env, info, denom)


    }
}


pub fn execute_receive(
    deps: DepsMut,
    info: MessageInfo,
    wrapper: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {

    let msg: ReceiveMsg = from_binary(&wrapper.msg)?;

    let balance = Balance::Cw20(Cw20CoinVerified {
        address: info.sender,
        amount: wrapper.amount,
    });

    let api = deps.api;
    match msg {
        ReceiveMsg::Limit(msg) => {
            ordergroup::start_limit(deps, msg, balance, api.addr_validate(&wrapper.sender)?)
        },
        ReceiveMsg::Smart(msg) => {
            ordergroup::start_smart(deps, msg, balance, api.addr_validate(&wrapper.sender)?)
        },
        ReceiveMsg::Grid(msg) => {
            ordergroup::start_grid(deps, msg, balance, api.addr_validate(&wrapper.sender)?)
        }
    }
}

pub fn execute_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    denom: Denom
) -> Result<Response, ContractError> {

    util::check_owner(deps.storage, info.sender.clone())?;
    let amount = util::get_token_amount(deps.querier, denom.clone(), env.contract.address.clone())?;
    let message = util::transfer_token_message(deps.querier, denom.clone(), amount, info.sender.clone())?;

    Ok(Response::new()
        .add_attribute("action", "withdraw")
        .add_attribute("amount", amount)
        .add_message(message)
    )
    
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} 
            => to_binary(&query_config(deps)?),

        QueryMsg::OrderAddresses {order_type, start_after, limit} 
            => to_binary(&query_order_addresses(deps, order_type, start_after, limit)?),
        QueryMsg::OrderForAddressIds { order_type, address } 
            => to_binary(&query_order_for_address_ids(deps, order_type, address)?),
        QueryMsg::Order { order_type, address, id } 
            => to_binary(&query_order(deps, order_type, address, id)?),
        QueryMsg::Orders { order_type, address} 
            => to_binary(&query_orders(deps, order_type, address)?),
        
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let cfg = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: cfg.owner,
    })
}



fn map_orders_count(
    item: StdResult<(Addr, (Vec<u64>, u64))>,
) -> StdResult<Addr> {
    item.map(|(address, (_list, _max_id))| {
        address
    })
}

pub fn query_order_addresses(
    deps: Deps,
    order_type: u64,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<OrderAddressesResponse> {
    let limit = limit.unwrap_or(util::DEFAULT_LIMIT).min(util::MAX_LIMIT) as usize;
    let addr = maybe_addr(deps.api, start_after)?;
    let start = addr.map(|addr| Bound::exclusive(addr.as_ref()));

    let mut store = LIMIT_ORDERS_COUNT;
    if order_type == 0u64 {
        store = LIMIT_ORDERS_COUNT;
    } else if order_type == 1u64 {
        store = SMART_ORDERS_COUNT;
    } else if order_type == 2u64 {
        store = GRID_ORDERS_COUNT;
    }
    let addresses:StdResult<Vec<_>> = store.clone()
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| map_orders_count(item))
        .collect();

    Ok(OrderAddressesResponse { addresses: addresses? })
}

pub fn query_order_for_address_ids(
    deps: Deps,
    order_type: u64,
    address: Addr,
) -> StdResult<OrderForAddressIdsResponse> {

    let mut store = LIMIT_ORDERS_COUNT;
    if order_type == 0u64 {
        store = LIMIT_ORDERS_COUNT;
    } else if order_type == 1u64 {
        store = SMART_ORDERS_COUNT;
    } else if order_type == 2u64 {
        store = GRID_ORDERS_COUNT;
    }
    let (ids, _max_id) = store.clone().load(deps.storage, address.clone())?;

    Ok(OrderForAddressIdsResponse { address, ids })
}

pub fn query_order(
    deps: Deps,
    order_type: u64,
    address: Addr,
    id: u64
) -> StdResult<OrderResponse> {

    if order_type == 0u64 {
        let limit_order = LIMIT_ORDERS.load(deps.storage, (address.clone(), id))?;
        return Ok(OrderResponse {
            address: address.clone(),
            id,
            limit_order: Some(limit_order),
            smart_order: None,
            grid_order: None
        });
    } else if order_type == 1u64 {
        let smart_order = SMART_ORDERS.load(deps.storage, (address.clone(), id))?;
        return Ok(OrderResponse {
            address: address.clone(),
            id,
            smart_order: Some(smart_order),
            limit_order: None,
            grid_order: None
        });
    } else if order_type == 2u64 {
        let grid_order = GRID_ORDERS.load(deps.storage, (address.clone(), id))?;
        return Ok(OrderResponse {
            address: address.clone(),
            id,
            grid_order: Some(grid_order),
            smart_order: None,
            limit_order: None
        });
    } else {
        return Ok(OrderResponse {
            address: address.clone(),
            id,
            grid_order: None,
            smart_order: None,
            limit_order: None
        });
    }
    
    
}


pub fn query_orders(
    deps: Deps,
    order_type: u64,
    address: Addr
) -> StdResult<OrdersResponse> {

    
    if order_type == 0u64 {
        let (ids, _max_id) = LIMIT_ORDERS_COUNT.load(deps.storage, address.clone())?;
        let mut list:Vec<LimitConfig> = vec![];
        for i in ids {
            list.push(LIMIT_ORDERS.load(deps.storage, (address.clone(), i))?);
        }
        return Ok(OrdersResponse {
            address: address.clone(),
            limit_orders: Some(list),
            smart_orders: None,
            grid_orders: None
        });
    } else if order_type == 1u64 {
        let (ids, _max_id) = SMART_ORDERS_COUNT.load(deps.storage, address.clone())?;
        let mut list:Vec<SmartConfig> = vec![];
        for i in ids {
            list.push(SMART_ORDERS.load(deps.storage, (address.clone(), i))?);
        }
        return Ok(OrdersResponse {
            address: address.clone(),
            smart_orders: Some(list),
            limit_orders: None,
            grid_orders: None
        });
    } else if order_type == 2u64 {
        let (ids, _max_id) = GRID_ORDERS_COUNT.load(deps.storage, address.clone())?;
        let mut list:Vec<GridConfig> = vec![];
        for i in ids {
            list.push(GRID_ORDERS.load(deps.storage, (address.clone(), i))?);
        }
        return Ok(OrdersResponse {
            address: address.clone(),
            grid_orders: Some(list),
            smart_orders: None,
            limit_orders: None
        });
    } else {
        return Ok(OrdersResponse {
            address: address.clone(),
            limit_orders: None,
            smart_orders: None,
            grid_orders: None
        });
    }
    
    
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

