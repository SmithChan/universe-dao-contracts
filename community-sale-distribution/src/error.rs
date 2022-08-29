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

    #[error("Not enough Tokens")]
    NotEnoughTokens {},

    #[error("Already Claimed All")]
    AlreadyClaimedAll {},

    #[error("No Staked")]
    NoStaked {},

    #[error("Not Created Unstaking")]
    NotCreatedUnstaking {},

    #[error("Not enough Reward")]
    NotEnoughReward { },

    #[error("Cannot migrate from different contract type: {previous_contract}")]
    CannotMigrate { previous_contract: String },

    #[error("Count {count}")]
    Count { count: u64 },
}
