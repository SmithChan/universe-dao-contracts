#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, from_binary,
    WasmMsg, WasmQuery, QueryRequest,Order, Addr, Storage, CosmosMsg, QuerierWrapper
};
use cw2::{get_contract_version, set_contract_version};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use cw_utils::{maybe_addr};
use cw_storage_plus::Bound;
use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, StakerListResponse, StakerInfo, StakerInput, UnstakingInfo, UnstakingResponse, ApyInfo,
    HistoryInfo, HistoryResponse, TreasuryConfigResponse, ReceiveMsg, StakerRecord
};
use crate::state::{
    Config, CONFIG, STAKERS, UNSTAKING, HISTORIES, APYS
};

// Version info, for migration info
const CONTRACT_NAME: &str = "universe_staking";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        owner: info.sender.clone(),
        verse_address: msg.verse_address,
        treasury_address: msg.treasury_address.clone(),
        sale_address: msg.treasury_address,
        stake_amount: vec![Uint128::zero(), Uint128::zero(), Uint128::zero()],
        lock_days: vec![7u64, 14u64, 28u64],
        enabled: true,
        last_apy_timestamp: 0u64,
        balance: Uint128::zero(),
        interval: msg.interval,
        
        fetch_from_treasury: false,
        apy: Uint128::from(100u128),
        multiple_1: Uint128::from(100u128),
        multiple_2: Uint128::from(100u128),
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
        ExecuteMsg::UpdateConfig { new_owner } => execute_update_config(deps, info, new_owner),
        ExecuteMsg::UpdateConstants { 
            verse_address, 
            treasury_address,
            sale_address,
            lock_days,
            interval
        } => execute_update_constants(deps, info, verse_address, treasury_address, sale_address, lock_days, interval),
        ExecuteMsg::UpdateEnabled { 
            enabled
        } => execute_update_enabled(deps, info, enabled),
        ExecuteMsg::UpdateFetchFromTreasury { 
            fetch_from_treasury
        } => execute_update_fetch_from_treasury(deps, info, fetch_from_treasury),
        ExecuteMsg::Rebase{ addresses } => execute_rebase(deps, env, info, addresses),
        ExecuteMsg::Receive(msg) => execute_receive(deps, env, info, msg),
        ExecuteMsg::CreateUnstake {unstake_amount, apy_type} => execute_create_unstake(deps, env, info, unstake_amount, apy_type),
        ExecuteMsg::FetchUnstake {apy_type, index} => execute_fetch_unstake(deps, env, info, apy_type, index),
        ExecuteMsg::AddStakers { stakers } => execute_add_stakers(deps, env, info, stakers),
        ExecuteMsg::RemoveStaker { address, apy_type } => execute_remove_staker(deps, info, address, apy_type),
        ExecuteMsg::RemoveAllStakers { } => execute_remove_all_stakers(deps, info),
        ExecuteMsg::SendVerse {address, amount} => execute_send_verse(deps, env, info, address, amount),
        ExecuteMsg::UpdateApy{ apy, multiple_1, multiple_2 } => execute_update_apy(deps, env, info, apy, multiple_1, multiple_2)
    }
}

pub fn rebase (
    storage: &mut dyn Storage,
    querier: QuerierWrapper,
    env: Env,
    address: Addr,
    apy_type: u64
) -> Result<Response, ContractError> {

    //update apy list
    let current_timestamp = env.block.time.seconds();
    let cfg = CONFIG.load(storage)?;
    let delta = current_timestamp / cfg.interval - cfg.last_apy_timestamp / cfg.interval;
    if delta > 0u64 {
        let apys: Vec<Uint128>;
        if cfg.fetch_from_treasury {
            let treasury_config: TreasuryConfigResponse =
            querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: cfg.treasury_address.clone().into(),
                msg: to_binary(&QueryMsg::Config {})?,
            }))?;

            apys = vec![treasury_config.apy, treasury_config.apy * treasury_config.multiple_1 / Uint128::from(100u128), treasury_config.apy * treasury_config.multiple_2 / Uint128::from(100u128)];
        } else {
            apys = vec![cfg.apy, cfg.apy * cfg.multiple_1 / Uint128::from(100u128), cfg.apy * cfg.multiple_2 / Uint128::from(100u128)];
        }
        
    
        APYS.save(storage, current_timestamp, &apys)?;
        CONFIG.update(storage, |mut exists| -> StdResult<_> {
            for i in 0..3 {
                for _j in 0..delta {
                    exists.stake_amount[i as usize] = exists.stake_amount[i as usize] * Uint128::from(apys[i as usize] / Uint128::from(100u128));
                }
            }
            exists.last_apy_timestamp = current_timestamp;
            Ok(exists)
        })?;
    }
    
    //update user's balance
    let mut arr = STAKERS.load(storage, address.clone()).unwrap_or(vec![]);

    if arr.len() == 0 {
        return Ok(Response::default());
    }
        
    for i in 0..arr.len() {
        let (mut amount, timestamp, apy_type_local) = arr[i];
        if apy_type != apy_type_local || amount == Uint128::zero() {
            continue;
        }
        
        let mut before_timestamp = timestamp;
        let mut offset;
        let start = Some(Bound::exclusive(timestamp));
        let apys:StdResult<Vec<_>> = APYS
            .range(storage, start, None, Order::Ascending)
            .map(|item| map_apys(item))
            .collect();
        for apy in apys.unwrap() {
            if apy.timestamp <= timestamp {
                before_timestamp = apy.timestamp;
                continue
            }
            offset = apy.timestamp / cfg.interval - before_timestamp / cfg.interval;

            while offset > 0u64 {
                amount = amount * Uint128::from(apy.apys[apy_type as usize]) / Uint128::from(100u128);
                offset -= 1u64;
            }
            before_timestamp = apy.timestamp;
        }
        arr[i] = (amount, before_timestamp, apy_type_local);
    }
    STAKERS.save(storage, address.clone(), &arr)?;

    Ok(Response::default())
}

pub fn add_history(
    storage: &mut dyn Storage,
    env: Env,
    address: Addr,
    is_staking: bool,
    amount: Uint128,
    apy_type: u64
) -> Result<Response, ContractError> {
    let mut history = HISTORIES.load(storage, address.clone()).unwrap_or(vec![]);

    history.push((amount, env.block.time.seconds(), is_staking, apy_type));
    HISTORIES.save(storage, address.clone(), &history)?;

    Ok(Response::default())
}


pub fn execute_update_apy(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    apy: Uint128,
    multiple_1: Uint128,
    multiple_2: Uint128
) -> Result<Response, ContractError> {
    // authorize owner
    check_owner(&deps, &info)?;
    
    CONFIG.update(deps.storage, |mut exists| -> StdResult<_> {
        exists.apy = apy;
        exists.multiple_1 = multiple_1;
        exists.multiple_2 = multiple_2;
        Ok(exists)
    })?;

    Ok(Response::new().add_attribute("action", "update_apy"))
}


pub fn execute_rebase(
    deps: DepsMut, 
    env: Env,
    info: MessageInfo, 
    addresses: Vec<Addr>
) -> Result<Response, ContractError> {
    
    // check_owner(&deps, &info)?;
    check_enabled(&deps, &info)?;

    for address in addresses.clone() {
        let arr = STAKERS.load(deps.storage, address.clone())?;
        for (amount, timestamp, apy_type) in arr {
            let cfg = CONFIG.load(deps.storage)?;
            if cfg.last_apy_timestamp > timestamp {
                rebase(deps.storage, deps.querier, env.clone(), address.clone(), apy_type)?;
            }
        }
    }
    
    return Ok(Response::new()
        .add_attributes(vec![
            attr("action", "rebase"),
            attr("count", addresses.clone().len().to_string()),
        ]));
    
}


pub fn execute_receive(
    deps: DepsMut, 
    env: Env,
    info: MessageInfo, 
    wrapper: Cw20ReceiveMsg
) -> Result<Response, ContractError> {
    
    check_enabled(&deps, &info)?;
    let mut cfg = CONFIG.load(deps.storage)?;
    let msg: ReceiveMsg = from_binary(&wrapper.msg)?;
    let user_addr = &deps.api.addr_validate(&wrapper.sender)?;
    
    if wrapper.amount == Uint128::zero() {
        return Err(ContractError::InvalidInput {});
    }
    if info.sender != cfg.verse_address {
        return Err(ContractError::UnacceptableToken {});
    }

    match msg {
        ReceiveMsg::Stake{ apy_type} => {
            // Update Amount
            rebase(deps.storage, deps.querier, env.clone(), user_addr.clone(), apy_type)?;
            cfg = CONFIG.load(deps.storage)?;
            let mut arr = STAKERS.load(deps.storage, user_addr.clone()).unwrap_or(vec![]);
            
            let mut exist = false;
            for i in 0..arr.len() {
                let (mut amount, timestamp, apy_type_local) = arr[i];
                if apy_type_local != apy_type {
                    continue;
                }
                exist = true;
                amount += wrapper.amount;
                arr[i] = (amount, timestamp, apy_type_local);
            }
            if !exist {
                arr.push((wrapper.amount, env.block.time.seconds(), apy_type));
            }
            
            STAKERS.save(deps.storage, user_addr.clone(), &arr)?;

            // Burn received VERSE
            let mut messages:Vec<CosmosMsg> = vec![];
            
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: cfg.verse_address.clone().into(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Burn {
                    amount: wrapper.amount
                })?,
            }));

            cfg.stake_amount[apy_type as usize] += wrapper.amount;
            CONFIG.save(deps.storage, &cfg)?;
            
            add_history(deps.storage, env, user_addr.clone(), true, wrapper.amount, apy_type)?;

            return Ok(Response::new()
                .add_messages(messages)
                .add_attributes(vec![
                    attr("action", "stake"),
                    attr("type", Uint128::from(apy_type as u64)),
                    attr("address", user_addr),
                    attr("amount", wrapper.amount)
                ]));
        },
        ReceiveMsg::Fund{} => {
            CONFIG.update(deps.storage, |mut exists| -> StdResult<_> {
                exists.balance += wrapper.amount;
                Ok(exists)
            })?;
            return Ok(Response::new()
                .add_attributes(vec![
                    attr("action", "fund"),
                    attr("amount", wrapper.amount)
                ])); 
        }
        
    }
    
}


pub fn execute_create_unstake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    unstake_amount: Uint128,
    apy_type: u64
) -> Result<Response, ContractError> {

    check_enabled(&deps, &info)?;

    if unstake_amount == Uint128::zero() {
        return Err(ContractError::InvalidInput {});
    }
    
    rebase(deps.storage, deps.querier, env.clone(), info.sender.clone(), apy_type)?;

    let mut cfg = CONFIG.load(deps.storage)?;
    let mut arr = STAKERS.load(deps.storage, info.sender.clone())?;
    
    let mut index = 0usize;
    for i in 0..arr.len() {
        let (_amount, _timestamp, apy_type_local) = arr[i];
        if apy_type_local != apy_type {
            continue;
        }
        index = i;
        break;
    }
    
    let (amount, timestamp, _apy_type_local) = arr[index];

    if amount < unstake_amount || cfg.stake_amount[apy_type as usize] < unstake_amount {
        return Err(ContractError::NotEnoughStake {});
    }

    arr[index] = (amount - unstake_amount, timestamp, apy_type);
    if amount == unstake_amount {
        arr.remove(index);
    }
    STAKERS.save(deps.storage, info.sender.clone(), &arr)?;

    let mut unstaking = UNSTAKING.load(deps.storage, info.sender.clone()).unwrap_or(vec![]);
    unstaking.push((unstake_amount, env.block.time.seconds() + cfg.lock_days[apy_type as usize] * cfg.interval, apy_type));
    UNSTAKING.save(deps.storage, info.sender.clone(), &unstaking)?;

    cfg.stake_amount[apy_type as usize] -= unstake_amount;
    CONFIG.save(deps.storage, &cfg)?;


    return Ok(Response::new()
        .add_attributes(vec![
            attr("action", "create_unstake"),
            attr("type", Uint128::from(apy_type as u64)),
            attr("address", info.sender.clone()),
            attr("unstake_amount",amount),
        ]));
}


pub fn execute_fetch_unstake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    apy_type: u64,
    index: u64
) -> Result<Response, ContractError> {

    check_enabled(&deps, &info)?;

    rebase(deps.storage, deps.querier, env.clone(), info.sender.clone(), apy_type)?;

    let cfg = CONFIG.load(deps.storage)?;
    
    let mut list = UNSTAKING.load(deps.storage, info.sender.clone()).unwrap_or(vec![]);

    if list.len() <= index as usize {
        return Err(ContractError::NotCreatedUnstaking {});
    }
    
    let (amount, timestamp, apy_type_local) = list[index as usize];

    if apy_type_local != apy_type {
        return Err(ContractError::IncorrectUnstaking {});
    }

    if timestamp > env.block.time.seconds() {
        return Err(ContractError::StillLocked {});
    }
    
    if amount > cfg.balance {
        return Err(ContractError::NotEnoughFund {});
    }
    CONFIG.update(deps.storage, |mut exists| -> StdResult<_> {
        exists.balance -= amount;
        Ok(exists)
    })?;
    
    list.remove(index as usize);
    UNSTAKING.save(deps.storage, info.sender.clone(), &list)?;

    add_history(deps.storage, env, info.sender.clone(), false, amount, apy_type)?;
    
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
            attr("action", "fetch_unstake"),
            attr("type", Uint128::from(apy_type as u64)),
            attr("address", info.sender.clone()),
            attr("unstake_amount", amount),
        ]));
}


pub fn execute_send_verse(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: Addr,
    amount: Uint128
) -> Result<Response, ContractError> {

    check_owner(&deps, &info)?;

    let cfg = CONFIG.load(deps.storage)?;
    if cfg.balance < amount {
        return Err(ContractError::NotEnoughFund {  });
    }
    let mut messages:Vec<CosmosMsg> = vec![];
        
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: cfg.verse_address.clone().into(),
        funds: vec![],
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: address.clone().into(),
            amount
        })?,
    }));

    return Ok(Response::new()
        .add_messages(messages)
        .add_attributes(vec![
            attr("action", "send_verse"),
            attr("address", address.clone()),
            attr("amount", amount),
        ]));
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

pub fn check_sale(
    deps: &DepsMut,
    info: &MessageInfo
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    if info.sender != cfg.sale_address {
        return Err(ContractError::Unauthorized {})
    }
    Ok(Response::new().add_attribute("action", "check_sale"))
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
    new_owner: String,
) -> Result<Response, ContractError> {
    // authorize owner
    check_owner(&deps, &info)?;
    
    let tmp_owner = deps.api.addr_validate(&new_owner)?;

    CONFIG.update(deps.storage, |mut exists| -> StdResult<_> {
        exists.owner = tmp_owner;
        Ok(exists)
    })?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

pub fn execute_update_constants(
    deps: DepsMut,
    info: MessageInfo,
    verse_address: Addr,
    treasury_address: Addr,
    sale_address: Addr,
    lock_days: Vec<u64>,
    interval: u64
) -> Result<Response, ContractError> {
    // authorize owner
    check_owner(&deps, &info)?;
    if lock_days.len() != 3 {
        return Err(ContractError::InvalidInput {});
    }
    CONFIG.update(deps.storage, |mut exists| -> StdResult<_> {
        exists.verse_address = verse_address;
        exists.treasury_address = treasury_address;
        exists.sale_address = sale_address;
        exists.lock_days = lock_days;
        exists.interval = interval;
        Ok(exists)
    })?;

    Ok(Response::new().add_attribute("action", "update_constants"))
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


pub fn execute_update_fetch_from_treasury(
    deps: DepsMut,
    info: MessageInfo,
    fetch_from_treasury: bool
) -> Result<Response, ContractError> {
    // authorize owner
    check_owner(&deps, &info)?;
    
    CONFIG.update(deps.storage, |mut exists| -> StdResult<_> {
        exists.fetch_from_treasury = fetch_from_treasury;
        Ok(exists)
    })?;

    Ok(Response::new().add_attribute("action", "update_fetch_from_treasury"))
}

pub fn execute_add_stakers(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    stakers: Vec<StakerInput>
) -> Result<Response, ContractError> {
    // authorize owner
    let a = check_sale(&deps, &info).is_ok();
    let b = check_owner(&deps, &info).is_ok();

    if !a && !b {
        return Err(ContractError::Unauthorized {});
    }
    for staker in stakers {
        
        let mut arr = STAKERS.load(deps.storage, staker.address.clone()).unwrap_or(vec![]);
        
        let mut exist = false;
        for i in 0..arr.len() {
            let (amount_local, timestamp_local, apy_type_local) = arr[i];
            if apy_type_local != staker.apy_type {
                continue;
            }
            exist = true;
            arr[i] = (amount_local + staker.amount, timestamp_local, apy_type_local);
            break;
        }
        if !exist {
            arr.push((staker.amount, env.block.time.seconds(), staker.apy_type));
        }
    
        STAKERS.save(deps.storage, staker.address.clone(), &arr)?;
    }
    
    Ok(Response::new().add_attribute("action", "add_stakers"))
}

pub fn execute_remove_staker(
    deps: DepsMut,
    info: MessageInfo,
    address: Addr,
    apy_type: u64
) -> Result<Response, ContractError> {
    // authorize owner
    check_owner(&deps, &info)?;
    
    let mut arr = STAKERS.load(deps.storage, address.clone()).unwrap_or(vec![]);
    if arr.len() == 0 {
        return Err(ContractError::NoStaked {  });
    }
    let mut index = arr.len();
    
    for i in 0..arr.len() {
        let (_amount, _timestamp, apy_type_local) = arr[i];
        if apy_type == apy_type_local {
            index = i;
            break;
        }
    }
    if index == arr.len() {
        return Err(ContractError::NoStaked {  });
    }
    arr.remove(index);
    STAKERS.save(deps.storage, address.clone(), &arr)?;
    
    Ok(Response::new().add_attribute("action", "remove_staker"))
}

pub fn execute_remove_all_stakers(
    deps: DepsMut,
    info: MessageInfo
) -> Result<Response, ContractError> {
    // authorize owner
    check_owner(&deps, &info)?;
    
    let stakers:StdResult<Vec<_>> = STAKERS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| map_staker(item))
        .collect();

    if stakers.is_err() {
        return Err(ContractError::Map2ListFailed {})
    }
    
    for item in stakers? {
        STAKERS.remove(deps.storage, item.address.clone());
    }
    
    Ok(Response::new().add_attribute("action", "remove_all_stakers"))
}




#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} 
            => to_binary(&query_config(deps)?),
        QueryMsg::Staker {address} 
            => to_binary(&query_staker(deps, address)?),
        QueryMsg::ListStakers {start_after, limit} 
            => to_binary(&query_list_stakers(deps, start_after, limit)?),
        QueryMsg::Unstaking {address} 
            => to_binary(&query_unstaking(deps, address)?),
        QueryMsg::Apys {}
            => to_binary(&query_apys(deps)?),
        QueryMsg::History {address}
            => to_binary(&query_history(deps, address)?)
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let cfg = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: cfg.owner,
        verse_address: cfg.verse_address,
        treasury_address: cfg.treasury_address,
        sale_address: cfg.sale_address,
        stake_amount: cfg.stake_amount,
        lock_days: cfg.lock_days,
        enabled: cfg.enabled,
        last_apy_timestamp: cfg.last_apy_timestamp,
        balance: cfg.balance,
        fetch_from_treasury: cfg.fetch_from_treasury,
        apy: cfg.apy,
        multiple_1: cfg.multiple_1,
        multiple_2: cfg.multiple_2
    })
}

// settings for pagination
const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;

fn query_staker(deps: Deps, address: Addr) -> StdResult<StakerInfo> {
    
    let arr = STAKERS.load(deps.storage, address.clone()).unwrap_or(vec![]);
    let mut ret = vec![];
    for i in 0..arr.len() {
        let (amount, timestamp, apy_type) = arr[i];
        ret.push(StakerRecord {
            amount,
            timestamp,
            apy_type
        });
    }

    Ok(StakerInfo {
        address,
        arr: ret
    })
}

fn map_staker(
    item: StdResult<(Addr, Vec<(Uint128, u64, u64)>)>,
) -> StdResult<StakerInfo> {
    item.map(|(address, arr)| {
        let mut ret = vec![];
        for i in 0..arr.len() {
            let (amount, timestamp, apy_type) = arr[i];
            ret.push(StakerRecord {
                amount,
                timestamp,
                apy_type
            });
        }
        StakerInfo {
            address,
            arr: ret
        }
    })
}

fn map_apys(
    item: StdResult<(u64, Vec<Uint128>)>,
) -> StdResult<ApyInfo> {
    item.map(|(timestamp, apys)| {
        ApyInfo {
            timestamp,
            apys
        }
    })
}

fn query_list_stakers(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<StakerListResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let addr = maybe_addr(deps.api, start_after)?;
    let start = addr.map(|addr| Bound::exclusive(addr));

    let stakers:StdResult<Vec<_>> = STAKERS
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| map_staker(item))
        .collect();

    Ok(StakerListResponse { stakers: stakers? })
}


fn query_unstaking(deps: Deps, address: Addr) -> StdResult<UnstakingResponse> {
    
    let unstaking = UNSTAKING.load(deps.storage, address.clone()).unwrap_or(vec![]);
    let mut unstaking_list = vec![];
    for (amount, timestamp, apy_type) in unstaking {
        unstaking_list.push(UnstakingInfo {
            amount,
            timestamp,
            apy_type
        });
    }
    
    Ok(UnstakingResponse {unstaking: unstaking_list})
}

fn query_apys(deps: Deps) -> StdResult<Vec<(u64, Vec<Uint128>)>> {
    
    let apys:StdResult<Vec<_>> = APYS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|item| map_apys(item))
        .collect();
    let mut ret:Vec<(u64, Vec<Uint128>)> = vec![];
    for apy in apys? {
        ret.push((apy.timestamp, apy.apys));
    }
    Ok(ret)
}

fn query_history(deps: Deps, address: Addr) -> StdResult<HistoryResponse> {
    let history_list = HISTORIES.load(deps.storage, address.clone()).unwrap_or(vec![]);
    let mut ret: Vec<HistoryInfo> = vec![];
    for (amount, timestamp, is_staking, apy_type) in history_list {
        ret.push(HistoryInfo{
            amount,
            timestamp,
            is_staking,
            apy_type
        });
    }
    Ok(HistoryResponse { history: ret})
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

