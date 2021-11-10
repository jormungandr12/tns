use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    SetRecord {
        node: Vec<u8>,
        owner: String,
        resolver: Option<String>,
        ttl: u64,
    },
    SetSubnodeOwner {
        node: Vec<u8>,
        label: Vec<u8>,
        owner: String,
    },
    SetOwner {
        node: Vec<u8>,
        owner: String,
    },
    SetResolver {
        node: Vec<u8>,
        resolver: Option<String>,
    },
    SetTTL {
        node: Vec<u8>,
        ttl: u64,
    },
    SetApprovalForAll {
        node: Vec<u8>,
        operator: String,
        approved: bool,
    },
    SetConfig {
        default_resolver: String,
        owner: String
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetRecord { name: String },
    GetRecordByNode { node: Vec<u8> },
    GetIsNodeOwner { node: Vec<u8>, address: String },
    GetIsApprovedForAll { owner: String, operator: String },
    GetConfig {},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RecordResponse {
    pub owner: Addr,
    pub resolver: Addr,
    pub ttl: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OperatorResponse {
    pub is_approve: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub default_resolver: Addr,
    pub owner: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
