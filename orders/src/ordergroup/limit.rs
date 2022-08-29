use cosmwasm_std::{
    DepsMut, Response, Uint128, Addr, CosmosMsg
};
use cw20::Balance;
use crate::error::ContractError;
use crate::msg::{
    LimitMsg, LimitConfig
};
use crate::state::{
    LIMIT_ORDERS_COUNT, LIMIT_ORDERS
};

use crate::util;

pub fn execute_start_limit(
    deps: DepsMut,
    msg: LimitMsg,
    balance: Balance,
    address: Addr,
) -> Result<Response, ContractError> {
    if balance.is_empty() {
        return Err(ContractError::EmptyBalance {});
    }
    // Update LIMIT_ORDERS_COUNT
    let (mut list, max_number) = LIMIT_ORDERS_COUNT.load(deps.storage, address.clone()).unwrap_or((vec![], 0));

    if list.len() as u64 >= util::MAX_ORDER {
        return Err(ContractError::MaxOrderCountExceed {});
    }

    let _first_token = util::check_token_and_pool(deps.querier, msg.token1_denom.clone(), msg.pool_address.clone())?;
    
    let token1_amount = util::get_amount_of_denom(balance, msg.token1_denom.clone())?;

    // Save current avg_buy_price
    let (token2_amount, token2_denom, messages) = util::get_swap_amount_and_denom_and_message(deps.querier, msg.pool_address.clone(), msg.token1_denom.clone(), token1_amount)?;

    let avg_buy_price = token1_amount * util::decimal() / token2_amount;
    let target_buy_price = avg_buy_price * (util::multiple() + Uint128::from(msg.take_profit_percentage)) / util::multiple();

    list.push(max_number);
    LIMIT_ORDERS_COUNT.save(deps.storage, address.clone(), &(list, max_number + 1))?;


    // Update LIMIT_ORDERS
    let limit_config = LimitConfig {
        msg: msg.clone(),
        avg_buy_price,
        target_buy_price,
        initial_token1_amount: token1_amount,
        token1_amount: Uint128::zero(),
        token2_amount,
        token2_denom,
        finished: false
    };

    LIMIT_ORDERS.save(deps.storage, (address.clone(), max_number), &limit_config)?;
    
    Ok(Response::new()
        .add_attribute("action", "start_limit")
        .add_attribute("address", address.clone().to_string())
        .add_messages(messages)
    )
}



pub fn execute_stop_limit(
    deps: DepsMut,
    address: Addr,
    id: u64
) -> Result<Response, ContractError> {
    Ok(execute_sync_limit(deps, address.clone(), Some(address.clone()), id, true)?)
}

pub fn execute_sync_limit(
    deps: DepsMut,
    caller: Addr, 
    address: Option<Addr>,
    id: u64,
    force_finish: bool
) -> Result<Response, ContractError> {

    let real_address;
    match address {
        Some(addr) => {real_address = addr.clone();},
        None => {real_address = caller.clone();}
    }

    if real_address != caller.clone() {
        util::check_owner(deps.storage, caller.clone())?;
    }
    
    let (mut list, _max_number) = LIMIT_ORDERS_COUNT.load(deps.storage, real_address.clone()).unwrap_or((vec![], 0));
    if !list.contains(&id) {
        return Err(ContractError::OrderNotExist {});
    }
    
    let mut limit_config = LIMIT_ORDERS.load(deps.storage, (real_address.clone(), id))?;
    
    if limit_config.finished {
        return Err(ContractError::AlreadyFinishedOrder {});
    }
    let (swap_amount, _other_denom, _message) = util::get_swap_amount_and_denom_and_message(deps.querier, limit_config.msg.pool_address.clone(), limit_config.msg.token1_denom.clone(), limit_config.initial_token1_amount)?;

    let current_buy_price = limit_config.initial_token1_amount * util::decimal() / swap_amount;

    // return Err(ContractError::DebugValue { value: current_buy_price});

    if current_buy_price > limit_config.target_buy_price || force_finish {
        let (index, _max_number) = list.iter().enumerate().find(|(_i, c)| c == &&id).unwrap_or((0, &0));
        list.remove(index);
        let mut messages: Vec<CosmosMsg> = vec![];
        
        let (swap_amount, _origin_denom, messages_swap) = util::get_swap_amount_and_denom_and_message(deps.querier, limit_config.msg.pool_address.clone(), limit_config.token2_denom.clone(), limit_config.token2_amount)?;
        
        for i in 0..messages_swap.len() {
            messages.push(messages_swap[i].clone());
        }

        //transfer to sender
        messages.push(util::transfer_token_message(deps.querier, limit_config.msg.token1_denom.clone(), swap_amount, real_address.clone())?);

        limit_config.finished = true;
        LIMIT_ORDERS.save(deps.storage, (real_address.clone(), id), &limit_config)?;

        return Ok(Response::new()
            .add_attribute("action", "sync_limit_success")
            .add_attribute("sender", real_address.to_string())
            .add_attribute("id", id.to_string())
            .add_messages(messages)
        );
    } else {
        return Ok(Response::new()
            .add_attribute("action", "sync_limit_waiting")
            .add_attribute("sender", real_address.to_string())
            .add_attribute("id", id.to_string())
        );
    }
    
}
