use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized { description: Option<String> },

    #[error("NotOwner: Sender is {sender}, but owner is {owner}.")]
    NotOwner {
        sender: String,
        owner: String
    },

    #[error("NotController: Sender {sender} is not controller.")]
    NotController {
        sender: String,
    },

    #[error("token_id already claimed")]
    Claimed {},

    #[error("Cannot set approval that is already expired")]
    Expired {},

    #[error("Not available")]
    NotAvailable {},

    #[error("Burn burn: {msg}")]
    Burned {
        msg: String,
    },

    #[error("BytesFormatError")]
    BytesFormatError {},

    #[error("IdAndNameNotMatch")]
    IdAndNameNotMatch {},

    #[error("NameAndHashNotMatch")]
    NameAndHashNotMatch {},
}
