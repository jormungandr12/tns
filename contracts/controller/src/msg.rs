use cosmwasm_std::{Addr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub registrar_address: String,
    pub max_commitment_age: u64,
    pub min_commitment_age: u64,
    pub min_registration_duration: u64,
    pub tier1_price: u64,
    pub tier2_price: u64,
    pub tier3_price: u64,
    pub enable_registration: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Commit {
        commitment: String,
    },
    Register {
        name: String,
        owner: String,
        duration: u64,
        secret: String,
        resolver: Option<String>,
        address: Option<String>,
    },
    OwnerRegister {
        name: String,
        owner: String,
        duration: u64,
        resolver: Option<String>,
        address: Option<String>,
    },
    SetConfig {
        max_commitment_age: u64,
        min_commitment_age: u64,
        min_registration_duration: u64,
        tier1_price: u64,
        tier2_price: u64,
        tier3_price: u64,
        registrar_address: String,
        owner: String,
        enable_registration: bool,
    },
    Withdraw {},
    Renew {
        name: String,
        duration: u64,
    },
    OwnerRenew {
        name: String,
        duration: u64,
    },
    SetEnableRegistration {
        enable_registration: bool,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Owner {},
    Registrar {},
    CommitmentTimestamp {
        commitment: String,
    },
    GetCommitment {
        name: String,
        owner: String,
        secret: String,
        resolver: Option<String>,
        address: Option<String>,
    },
    RentPrice {
        name: String,
        duration: u64,
    },
    MaxCommitmentAge {},
    MinCommitmentAge {},
    MinRegistrationDuration {},
    IsValidName {
        name: String,
    },
    GetTokenId {
        name: String,
    },
    GetNodehash {
        name: String,
    },
    GetNodeInfo {
        name: String,
    },
    GetPrice {},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetCommitmentResponse {
    pub commitment: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CommitmentTimestampResponse {
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RentPriceResponse {
    pub price: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MaxCommitmentAgeResponse {
    pub age: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MinCommitmentAgeResponse {
    pub age: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MinRegistrationDurationResponse {
    pub duration: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IsValidNameResponse {
    pub is_valid_name: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenIdResponse {
    pub token_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NodehashResponse {
    pub node: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NodeInfoResponse {
    pub label: Vec<u8>,
    pub token_id: String,
    pub node: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OwnerResponse {
    pub owner: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RegistrarResponse {
    pub registrar_address: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PriceResponse {
    pub tier1_price: u64,
    pub tier2_price: u64,
    pub tier3_price: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
