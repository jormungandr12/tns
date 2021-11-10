use cosmwasm_std::CanonicalAddr;
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub max_commitment_age: u64,
    pub min_commitment_age: u64,
    pub min_registration_duration: u64,
    pub tier1_price: u64,
    pub tier2_price: u64,
    pub tier3_price: u64,
    pub enable_registration: bool,
    pub registrar_address: CanonicalAddr,
    pub owner: CanonicalAddr,
}

pub const REGISTER_FEE_DENOM: &str = "uusd";
pub const CONFIG: Item<Config> = Item::new("CONFIG");
pub const COMMITMENTS: Map<String, u64> = Map::new("COMMITMENTS");
