use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("NotOwner: Sender is {sender}, but owner is {owner}.")]
    NotOwner {
        sender: String,
        owner: String
    },

    #[error("NotNodeOwner: Sender {sender} is not node owner of {node}.")]
    NotNodeOwner {
        sender: String,
        node: String
    },
}
