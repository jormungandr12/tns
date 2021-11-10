use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub interface_id: u64,
    pub registry_address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    SetAddress {
        node: Vec<u8>,
        coin_type: u64,
        address: String,
    },
    SetTerraAddress {
        node: Vec<u8>,
        address: String,
    },
    SetTextData {
        node: Vec<u8>,
        key: String,
        value: String,
    },
    SetContentHash {
        node: Vec<u8>,
        hash: Vec<u8>,
    },
    SetConfig {
        interface_id: u64,
        registry_address: String,
        owner: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetAddress { node: Vec<u8>, coin_type: u64 },
    GetTextData { node: Vec<u8>, key: String },
    GetTerraAddress { node: Vec<u8> },
    GetContentHash { node: Vec<u8> },
    GetConfig {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AddressResponse {
    pub address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TextDataResponse {
    pub data: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContentHashResponse {
    pub hash: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub interface_id: u64,
    pub registry_address: Addr,
    pub owner: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
