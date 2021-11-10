use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::CanonicalAddr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub default_resolver: CanonicalAddr,
    pub owner: CanonicalAddr,
}

pub const CONFIG: Item<Config> = Item::new("CONFIG");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Record {
    pub owner: CanonicalAddr,
    pub resolver: CanonicalAddr,
    pub ttl: u64,
}

pub const RECORDS: Map<Vec<u8>, Record> = Map::new("RECORDS");

pub const OPERATORS: Map<(Vec<u8>, Vec<u8>), bool> = Map::new("OPERATORS");
