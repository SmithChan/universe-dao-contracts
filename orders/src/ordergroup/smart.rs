use cosmwasm_std::{
    DepsMut, Response, Uint128, Addr, CosmosMsg
};
use cw20::Balance;
use crate::error::ContractError;
use crate::msg::{
    SmartMsg, SmartConfig
};
use crate::state::{
    SMART_ORDERS_COUNT, SMART_ORDERS
};

use crate::util;

pub fn execute_start_smart(
    deps: DepsMut,
    msg: SmartMsg,
    balance: Balance,
    address: Addr,
) -> Result<Response, ContractError> {
    if balance.is_empty() {
        return Err(ContractError::EmptyBalance {});
    }
    // Update SMART_ORDERS_COUNT
    let (mut list, max_number) = SMART_ORDERS_COUNT.load(deps.storage, address.clone()).unwrap_or((vec![], 0));

    if list.len() as u64 >= util::MAX_ORDER {
        return Err(ContractError::MaxOrderCountExceed {});
    }

    let mut messages: Vec<CosmosMsg> = vec![];
    let _first_token = util::check_token_and_pool(deps.querier, msg.token1_denom.clone(), msg.pool_address.clone())?;
    let token1_amount = util::get_amount_of_denom(balance, msg.token1_denom.clone())?;

    //check if token1_amount is greater than the amount for the total dca steps
    let mut tot_steps = 1u64;
    let mut mul = msg.dca_order_size_multiplier;
    for _i in 0..msg.num_dca_orders {
        tot_steps += mul;
        mul *= mul;
    }

    if Uint128::from(tot_steps) * msg.initial_token1_amount > token1_amount {
        return Err(ContractError::InsufficientAmountForSmartOrder {});
    } else {
        messages.push(util::transfer_token_message(deps.querier, msg.token1_denom.clone(), token1_amount - Uint128::from(tot_steps) * msg.initial_token1_amount, address.clone())?);
    }

    // Save current avg_buy_price
    let (token2_amount, token2_denom, mut messages_swap) = util::get_swap_amount_and_denom_and_message(deps.querier, msg.pool_address.clone(), msg.token1_denom.clone(), msg.initial_token1_amount)?;
    messages.append(&mut messages_swap);

    let avg_buy_price = token1_amount * util::decimal() / token2_amount;
    let target_buy_price = avg_buy_price * (util::multiple() + Uint128::from(msg.take_profit_percentage)) / util::multiple();

    list.push(max_number);
    SMART_ORDERS_COUNT.save(deps.storage, address.clone(), &(list, max_number + 1))?;


    // Update SMART_ORDERS
    //make dca_prices, dca_amounts list
    let mut dca_prices:Vec<Uint128> = vec![];
    let mut dca_amounts:Vec<Uint128> = vec![];

    let mut mul_price = 1u64;
    let mut mul_amount = Uint128::from(1u128);
    let mut start_val = util::multiple();
    for _i in 0..msg.num_dca_orders {
        mul_price *= msg.dca_step_multiplier;
        dca_prices.push( avg_buy_price * (start_val - Uint128::from(msg.dca_step * mul_price)) / util::multiple() );
        start_val -= Uint128::from(msg.dca_step * mul_price);

        mul_amount *= Uint128::from(msg.dca_order_size_multiplier);
        dca_amounts.push( mul_amount * msg.dca_order_size);
        
    }

    let smart_config = SmartConfig {
        msg: msg.clone(),
        avg_buy_price,
        target_buy_price,
        token1_amount: token1_amount - msg.initial_token1_amount,
        token2_amount,
        token2_denom,
        finished: false,
        dca_prices,
        dca_amounts,
        current_dca_point: 0u64
    };

    SMART_ORDERS.save(deps.storage, (address.clone(), max_number), &smart_config)?;
    
    Ok(Response::new()
        .add_attribute("action", "start_smart")
        .add_attribute("address", address.clone().to_string())
        .add_messages(messages)
    )
}



pub fn execute_stop_smart(
    deps: DepsMut,
    address: Addr,
    id: u64
) -> Result<Response, ContractError> {
    Ok(execute_sync_smart(deps, address.clone(), Some(address.clone()), id, true)?)
}

pub fn execute_sync_smart(
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
    
    let (mut list, _max_number) = SMART_ORDERS_COUNT.load(deps.storage, real_address.clone()).unwrap_or((vec![], 0));
    if !list.contains(&id) {
        return Err(ContractError::OrderNotExist {});
    }
    let mut smart_config = SMART_ORDERS.load(deps.storage, (real_address.clone(), id))?;

    if smart_config.finished {
        return Err(ContractError::AlreadyFinishedOrder {});
    }

    //check the current_dca_point and do swap Juno->Atom while the current buy price is larger than dca_price

    let mut messages: Vec<CosmosMsg> = vec![];
    while smart_config.current_dca_point < smart_config.msg.num_dca_orders {
        let (swap_amount, _other_denom, mut messages_swap) = util::get_swap_amount_and_denom_and_message(deps.querier, smart_config.msg.pool_address.clone(), smart_config.msg.token1_denom.clone(), smart_config.dca_amounts[smart_config.current_dca_point as usize])?;
        let buy_price = smart_config.dca_amounts[smart_config.current_dca_point as usize] * util::decimal() / swap_amount;

        if buy_price < smart_config.dca_prices[smart_config.current_dca_point as usize] {
            // do the swap
            messages.append(&mut messages_swap);
    
            smart_config.token1_amount -= smart_config.dca_amounts[smart_config.current_dca_point as usize];
            smart_config.token2_amount += swap_amount;

            smart_config.current_dca_point += 1u64;
        } else {
            break;
        }
    }
    let (swap_amount, _origin_denom, mut messages_swap) = util::get_swap_amount_and_denom_and_message(deps.querier, smart_config.msg.pool_address.clone(), smart_config.token2_denom.clone(), smart_config.token2_amount)?;
    
    let mut action = String::from("sync_smart_waiting");
    //check if the ATOM->swap rate is larger than avg_buy_price or force_finish
    if swap_amount * util::decimal() / smart_config.token2_amount >= smart_config.target_buy_price || force_finish {
        let (index, _max_number) =list.iter().enumerate().find(|(_i, c)| c == &&id).unwrap_or((0, &0));
        list.remove(index);

        messages.append(&mut messages_swap);

        //transfer to sender
        messages.push(util::transfer_token_message(deps.querier, smart_config.msg.token1_denom.clone(), swap_amount + smart_config.token1_amount, real_address.clone())?);

        smart_config.finished = true;
        smart_config.token1_amount += swap_amount;
        smart_config.token2_amount = Uint128::zero();
        action = String::from("sync_smart_success");

    }
    SMART_ORDERS.save(deps.storage, (real_address.clone(), id), &smart_config)?;

    return Ok(Response::new()
        .add_attribute("action", action)
        .add_attribute("sender", real_address.to_string())
        .add_attribute("id", id.to_string())
        .add_messages(messages)
    );
}
