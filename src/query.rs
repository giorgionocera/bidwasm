use cosmwasm_std::{Deps, StdError, StdResult, Uint128};

use crate::{
    msg::BidResp,
    state::{Status, BIDS, STATE},
};

pub fn total_bid(deps: Deps, address: String) -> StdResult<Uint128> {
    let address = deps.api.addr_validate(&address)?;

    Ok(BIDS.may_load(deps.storage, &address)?.unwrap_or_default())
}

pub fn highest_bid(deps: Deps) -> StdResult<BidResp> {
    match STATE.load(deps.storage)?.highest_bid {
        Some((address, amount)) => Ok(BidResp { address, amount }),
        None => Err(StdError::not_found("The auction has not any bid")),
    }
}

pub fn is_closed(deps: Deps) -> StdResult<bool> {
    match STATE.load(deps.storage)?.current_status {
        Status::Closed => Ok(true),
        _ => Ok(false),
    }
}

pub fn winner(deps: Deps) -> StdResult<BidResp> {
    if STATE.load(deps.storage)?.current_status == Status::Open {
        return Err(StdError::generic_err("The auction is yet open"));
    }
    match STATE.load(deps.storage)?.highest_bid {
        Some((address, amount)) => Ok(BidResp { address, amount }),
        None => Err(StdError::not_found("The auction has not any bid")),
    }
}
