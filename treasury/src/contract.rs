#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, Coin, BankMsg,
     Addr, Storage
};
use cw2::{get_contract_version, set_contract_version};
use crate::error::ContractError;
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg, HistoryInfo, HistoryResponse
};
use crate::state::{
    Config, CONFIG, HISTORIES
};

// Version info, for migration info
const CONTRACT_NAME: &str = "universe_treasury";
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
        treasury_amount: Uint128::zero(),
        treasury_denom: String::from("ujunox"),
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
            treasury_denom
        } => execute_update_constants(deps, info, treasury_denom),
        ExecuteMsg::AddFund { } => execute_add_fund(deps, env, info),
        ExecuteMsg::RemoveFund{ amount } => execute_remove_fund(deps, env, info, amount),
        ExecuteMsg::RemoveAll {} => execute_remove_all(deps, env, info),
        ExecuteMsg::UpdateApy{ apy, multiple_1, multiple_2 } => execute_update_apy(deps, env, info, apy, multiple_1, multiple_2)
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

pub fn add_history(
    storage: &mut dyn Storage,
    env: Env,
    owner: Addr,
    address: Addr,
    is_add: bool,
    amount: Uint128
) -> Result<Response, ContractError> {
    let exists = HISTORIES.may_load(storage, address.clone())?;
    let mut history = vec![];
    if exists.is_some() {
        history = exists.unwrap();
    }

    history.push((address, env.block.height, env.block.time.seconds(), is_add, amount));
    HISTORIES.save(storage, owner.clone(), &history)?;

    Ok(Response::default())
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

pub fn execute_add_fund(
    deps: DepsMut,
    env: Env,
    info: MessageInfo
) -> Result<Response, ContractError> {

    let mut cfg = CONFIG.load(deps.storage)?;
    let mut funds = Coin {
        amount: Uint128::new(0),
        denom: cfg.treasury_denom.clone(),
    };

    for coin in &info.funds {
        if coin.denom == cfg.treasury_denom {
            funds = Coin {
                amount: funds.amount + coin.amount,
                denom: funds.denom,
            }
        }
    }

    if funds.amount == Uint128::new(0) {
        return Err(ContractError::InvalidInput {});
    }

    cfg.treasury_amount += funds.amount;
    CONFIG.save(deps.storage, &cfg)?;

    add_history(deps.storage, env.clone(), env.clone().contract.address.clone(), info.sender.clone(), true, funds.amount)?;

    return Ok(Response::new()
        .add_attributes(vec![
            attr("action", "add_fund"),
            attr("address", info.sender.clone()),
            attr("amount", funds.amount),
        ]));
}


pub fn execute_remove_fund(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128
) -> Result<Response, ContractError> {

    check_owner(&deps, &info)?;

    let mut cfg = CONFIG.load(deps.storage)?;
    if cfg.treasury_amount < amount {
        return Err(ContractError::NotEnoughCoins {});
    }

    let funds = Coin {
        amount,
        denom: cfg.treasury_denom.clone(),
    };

    let transfer_bank_msg = BankMsg::Send {
        to_address: info.sender.clone().into(),
        amount: vec![funds]
    };

    cfg.treasury_amount -= amount;
    CONFIG.save(deps.storage, &cfg)?;
    
    add_history(deps.storage, env.clone(), env.clone().contract.address.clone(), info.sender.clone(), false, amount)?;
    
    return Ok(Response::new()
        .add_message(transfer_bank_msg)
        .add_attributes(vec![
            attr("action", "remove_fund"),
            attr("address", info.sender.clone()),
            attr("amount", amount),
        ]));
}

pub fn execute_remove_all(
    deps: DepsMut,
    env: Env,
    info: MessageInfo
) -> Result<Response, ContractError> {

    check_owner(&deps, &info)?;

    let mut cfg = CONFIG.load(deps.storage)?;
    cfg.treasury_amount = Uint128::zero();
    CONFIG.save(deps.storage, &cfg)?;

    let address = env.contract.address.clone();
    let ret = deps.querier.query_balance(address, cfg.treasury_denom.clone())?;
    let amount = ret.amount;
    if amount == Uint128::zero() {
        return Err(ContractError::NotEnoughCoins {});
    }

    let funds = Coin {
        amount,
        denom: cfg.treasury_denom.clone(),
    };

    let transfer_bank_msg = BankMsg::Send {
        to_address: info.sender.clone().into(),
        amount: vec![funds]
    };

    return Ok(Response::new()
        .add_message(transfer_bank_msg)
        .add_attributes(vec![
            attr("action", "remove_all"),
            attr("address", info.sender.clone()),
            attr("amount", amount),
        ]));
}

pub fn execute_update_constants(
    deps: DepsMut,
    info: MessageInfo,
    treasury_denom: String
) -> Result<Response, ContractError> {
    // authorize owner
    check_owner(&deps, &info)?;
    
    CONFIG.update(deps.storage, |mut exists| -> StdResult<_> {
        exists.treasury_denom = treasury_denom;
        Ok(exists)
    })?;

    Ok(Response::new().add_attribute("action", "update_constants"))
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



#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} 
            => to_binary(&query_config(deps)?),
        QueryMsg::History {}
            => to_binary(&query_history(deps, env)?)
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let cfg = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        owner: cfg.owner,
        treasury_amount: cfg.treasury_amount,
        treasury_denom: cfg.treasury_denom,
        apy: cfg.apy,
        multiple_1: cfg.multiple_1,
        multiple_2: cfg.multiple_2
    })
}

fn query_history(deps: Deps, env: Env) -> StdResult<HistoryResponse> {
    let history_list = HISTORIES.load(deps.storage, env.contract.address.clone()).unwrap_or(vec![]);
    let mut ret: Vec<HistoryInfo> = vec![];
    for (address, height, timestamp, is_add, amount) in history_list {
        ret.push(HistoryInfo {
            address,
            height,
            timestamp,
            is_add,
            amount
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

