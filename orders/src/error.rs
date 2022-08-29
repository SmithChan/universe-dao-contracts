use cosmwasm_std::{StdError, Uint128};
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
    
    #[error("Max Order Count Exceed")]
    MaxOrderCountExceed {},

    #[error("Insufficient amount for Smart order")]
    InsufficientAmountForSmartOrder {},

    #[error("Insufficient amount for Grid order")]
    InsufficientAmountForGridOrder {},

    #[error("OrderNotExist")]
    OrderNotExist {},

    #[error("AlreadyFinishedOrder")]
    AlreadyFinishedOrder {},

    #[error("Amount of the native coin inputed is zero")]
    NativeInputZero {},

    #[error("Amount of the cw20 coin inputed is zero")]
    Cw20InputZero {},

    #[error("Token type mismatch")]
    TokenTypeMismatch {},

    #[error("The pool does not contain the input token")]
    PoolAndTokenMismatch {},

    #[error("InvalidInput")]
    InvalidInput {},

    #[error("Send some coins to create an order")]
    EmptyBalance {},

    #[error("Debug {value}")]
    DebugValue { value: Uint128},

    #[error("Cannot migrate from different contract type: {previous_contract}")]
    CannotMigrate { previous_contract: String },

}
