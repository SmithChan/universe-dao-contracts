use cosmwasm_std::{StdError};
use hex::FromHexError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Hex(#[from] FromHexError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Disabled")]
    Disabled {},

    #[error("InvalidInput")]
    InvalidInput {},

    #[error("Not Reward or Stake token")]
    UnacceptableToken {},

    #[error("Not enough Stake")]
    NotEnoughStake {},

    #[error("Still Locked")]
    StillLocked {},

    #[error("No Staked")]
    NoStaked {},

    #[error("Not Created Unstaking")]
    NotCreatedUnstaking {},

    #[error("IncorrectUnstaking")]
    IncorrectUnstaking {},

    #[error("Not enough Fund")]
    NotEnoughFund { },

    #[error("Map2List failed")]
    Map2ListFailed {},

    #[error("Cannot migrate from different contract type: {previous_contract}")]
    CannotMigrate { previous_contract: String },
    
    #[error("Count {count}")]
    Count { count: u64 },
}
