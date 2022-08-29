use cosmwasm_std::{
    DepsMut, Response, Uint128, Addr, CosmosMsg
};
use cw20::Balance;
use crate::error::ContractError;
use crate::msg::{
    GridMsg, GridConfig
};
use crate::state::{
    GRID_ORDERS_COUNT, GRID_ORDERS
};

use crate::util;

pub fn execute_start_grid(
    deps: DepsMut,
    msg: GridMsg,
    balance: Balance,
    address: Addr,
) -> Result<Response, ContractError> {
    if balance.is_empty() {
        return Err(ContractError::EmptyBalance {});
    }
    // Update GRID_ORDERS_COUNT
    let (mut list, max_number) = GRID_ORDERS_COUNT.load(deps.storage, address.clone()).unwrap_or((vec![], 0));

    if list.len() as u64 >= util::MAX_ORDER {
        return Err(ContractError::MaxOrderCountExceed {});
    }

    let mut messages: Vec<CosmosMsg> = vec![];
    
    // UPDATE GRID_ORDERS
    let _first_token = util::check_token_and_pool(deps.querier, msg.token1_denom.clone(), msg.pool_address.clone())?;
    let mut token1_amount = util::get_amount_of_denom(balance, msg.token1_denom.clone())?;

    if token1_amount < msg.total_amount {
        return Err(ContractError::InsufficientAmountForGridOrder {});
    } else if token1_amount > msg.total_amount {
        messages.push(util::transfer_token_message(deps.querier, msg.token1_denom.clone(), token1_amount - msg.total_amount, address.clone())?);
        token1_amount = msg.total_amount;
    }
    list.push(max_number);
    GRID_ORDERS_COUNT.save(deps.storage, address.clone(), &(list, max_number + 1))?;

    // Do the initial swap
    let first_swap_amount = token1_amount / Uint128::from(2u128);
    let (token2_amount, token2_denom, mut messages_swap) = util::get_swap_amount_and_denom_and_message(deps.querier, msg.pool_address.clone(), msg.token1_denom.clone(), first_swap_amount)?;
    messages.append(&mut messages_swap);
    let avg_buy_price = first_swap_amount * util::decimal() / token2_amount;

    // Update GRID_ORDERS
    //make dca_prices, dca_amounts list
    let mut sell_prices:Vec<Uint128> = vec![];
    let mut buy_prices:Vec<Uint128> = vec![];
    
    let delta = msg.price_range_percentage / msg.num_grid_pairs;
    for i in 0..msg.num_grid_pairs {
        sell_prices.push(avg_buy_price * (util::multiple() + Uint128::from(delta * (i + 1))) / util::multiple() );
        buy_prices.push(avg_buy_price * (util::multiple() - Uint128::from(delta * (i + 1))) / util::multiple() );
        
    }

    let grid_config = GridConfig {
        msg: msg.clone(),
        token2_denom,
        buy_prices,
        sell_prices,
        order_amount: (msg.total_amount - first_swap_amount) / Uint128::from(msg.num_grid_pairs),
        finished: false,
        buy_step: 0u64,
        sell_step: 0u64,
        token1_amount: token1_amount - first_swap_amount,
        token2_amount
    };

    GRID_ORDERS.save(deps.storage, (address.clone(), max_number), &grid_config)?;
    
    Ok(Response::new()
        .add_attribute("action", "start_grid")
        .add_attribute("address", address.clone().to_string())
        .add_messages(messages)
    )
}



pub fn execute_stop_grid(
    deps: DepsMut,
    address: Addr,
    id: u64
) -> Result<Response, ContractError> {
    Ok(execute_sync_grid(deps, address.clone(), Some(address.clone()), id, true)?)
}

pub fn execute_sync_grid(
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
    
    let (mut list, _max_number) = GRID_ORDERS_COUNT.load(deps.storage, real_address.clone()).unwrap_or((vec![], 0));
    if !list.contains(&id) {
        return Err(ContractError::OrderNotExist {});
    }
    let mut grid_config = GRID_ORDERS.load(deps.storage, (real_address.clone(), id))?;

    if grid_config.finished {
        return Err(ContractError::AlreadyFinishedOrder {});
    }

    //check the current_dca_point and do swap Juno->Atom while the current buy price is larger than dca_price

    let mut messages: Vec<CosmosMsg> = vec![];
    //sell atom

    while grid_config.buy_step < grid_config.msg.num_grid_pairs {
        let (swap_amount, _other_denom, mut messages_swap) = util::get_swap_amount_and_denom_and_message(deps.querier, grid_config.msg.pool_address.clone(), grid_config.msg.token1_denom.clone(), grid_config.order_amount)?;
        let price = grid_config.order_amount * util::decimal() / swap_amount;

        if price <= grid_config.buy_prices[grid_config.buy_step as usize] {
            // do the swap
            messages.append(&mut messages_swap);
            grid_config.token1_amount -= grid_config.order_amount;
            grid_config.token2_amount += swap_amount;

            grid_config.buy_step += 1u64;
        } else {
            break;
        }
    }

    while grid_config.sell_step < grid_config.msg.num_grid_pairs {
        let (swap_amount, _other_denom, _temp_message) = util::get_swap_amount_and_denom_and_message(deps.querier, grid_config.msg.pool_address.clone(), grid_config.msg.token1_denom.clone(), grid_config.order_amount)?;

        let (token1_swap_amount, _token1_denom, mut messages_swap) = util::get_swap_amount_and_denom_and_message(deps.querier, grid_config.msg.pool_address.clone(), grid_config.token2_denom.clone(), swap_amount)?;

        let price = token1_swap_amount * util::decimal() / swap_amount;

        if price >= grid_config.sell_prices[grid_config.sell_step as usize] {
            // do the swap
            messages.append(&mut messages_swap);
            grid_config.token1_amount += token1_swap_amount;
            grid_config.token2_amount -= swap_amount;

            grid_config.sell_step += 1u64;
        } else {
            break;
        }
    }
    
    let mut action = String::from("sync_grid_waiting");
    //check if the ATOM->swap rate is larger than avg_buy_price
    if force_finish {
        let (index, _max_number) =list.iter().enumerate().find(|(_i, c)| c == &&id).unwrap_or((0, &0));
        list.remove(index);
        // Do final swap
        let (token1_swap_amount, _token1_denom, mut messages_swap) = util::get_swap_amount_and_denom_and_message(deps.querier, grid_config.msg.pool_address.clone(), grid_config.token2_denom.clone(), grid_config.token2_amount)?;
        messages.append(&mut messages_swap);
        
        //transfer to sender
        messages.push(util::transfer_token_message(deps.querier, grid_config.msg.token1_denom.clone(), token1_swap_amount + grid_config.token1_amount, real_address.clone())?);

        grid_config.finished = true;
        grid_config.token1_amount += token1_swap_amount;
        grid_config.token2_amount = Uint128::zero();
        action = String::from("sync_grid_success");

    }
    GRID_ORDERS.save(deps.storage, (real_address.clone(), id), &grid_config)?;

    return Ok(Response::new()
        .add_attribute("action", action)
        .add_attribute("sender", real_address.to_string())
        .add_attribute("id", id.to_string())
        .add_messages(messages)
    );
}
