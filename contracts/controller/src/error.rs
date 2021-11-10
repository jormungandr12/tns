use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized { description: Option<String> },

    #[error("NotOwner: Sender is {sender}, but owner is {owner}.")]
    NotOwner { sender: String, owner: String },

    #[error("RecommitTooEarly: You can recommit again after {commit_expired}. Current time is {current}.")]
    RecommitTooEarly { commit_expired: u64, current: u64 },

    #[error("CommitmentIsTooEarlyOrExpired: The commitment matures at {commit_matured} and expires at {commit_expired}. Current time is {current}.")]
    CommitmentIsTooEarlyOrExpired {
        commit_expired: u64,
        commit_matured: u64,
        current: u64,
    },

    #[error("ConsumeNonexistCommitment: The commitment {commitment} does not exist.")]
    ConsumeNonexistCommitment { commitment: String },

    #[error("UnavailabledName")]
    UnavailabledName {},

    #[error("InvalidName")]
    InvalidName {},

    #[error("NameTooShort")]
    NameTooShort {},

    #[error("DurationTooShort")]
    DurationTooShort {
        input_duration: u64,
        min_duration: u64,
    },

    #[error("InsufficientFund")]
    InsufficientFund { amount: Uint128, required: Uint128 },

    #[error("RegistrationDisabled")]
    RegistrationDisabled {},

    #[error("BadRequest")]
    BadRequest { msg: String },
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
