use crate::error::ContractError;
use crate::state::CONTENT_HASH;
use crate::state::TEXT_DATA;
use crate::state::{ADDRESSES, CONFIG};
use cosmwasm_std::{
    to_binary, Deps, DepsMut, Env, MessageInfo, QueryRequest, Response, StdResult, WasmQuery,
};
use cw_storage_plus::U64Key;
use tns::registry::QueryMsg as RegistryQueryMsg;
use tns::resolver::{AddressResponse, ConfigResponse, ContentHashResponse, TextDataResponse};

const LUNA_COIN_TYPE: u64 = 0x8000014a;

fn only_owner(deps: Deps, info: &MessageInfo) -> Result<bool, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let sender = deps.api.addr_canonicalize(info.sender.as_str())?;
    if sender != config.owner {
        return Err(ContractError::NotOwner {
            sender: info.sender.to_string(),
            owner: deps.api.addr_humanize(&config.owner)?.to_string(),
        });
    }
    Ok(true)
}

pub fn only_authorized(
    deps: &DepsMut,
    info: &MessageInfo,
    node: &Vec<u8>,
) -> Result<bool, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let registry_address = deps
        .api
        .addr_humanize(&config.registry_address)?
        .to_string();
    let is_node_owner: bool = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: registry_address.clone(),
        msg: to_binary(&RegistryQueryMsg::GetIsNodeOwner {
            node: node.clone(),
            address: info.sender.to_string(),
        })?,
    }))?;

    if is_node_owner {
        return Ok(true);
    }
    return Err(ContractError::NotNodeOwner {
        sender: info.sender.to_string(),
        node: format!("{:?}", node.clone()),
    });
}

pub fn set_address(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    node: Vec<u8>,
    coin_type: u64,
    address: String,
) -> Result<Response, ContractError> {
    only_authorized(&deps, &info, &node)?;
    ADDRESSES.save(deps.storage, (node, U64Key::from(coin_type)), &address)?;
    Ok(Response::default())
}

pub fn set_terra_address(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    node: Vec<u8>,
    address: String,
) -> Result<Response, ContractError> {
    let terra_address = deps.api.addr_validate(address.as_str())?;
    return set_address(
        deps,
        env,
        info,
        node,
        LUNA_COIN_TYPE,
        terra_address.to_string(),
    );
}

pub fn query_address(
    deps: Deps,
    _env: Env,
    node: Vec<u8>,
    coin_type: u64,
) -> StdResult<AddressResponse> {
    let address = ADDRESSES.load(deps.storage, (node, U64Key::from(coin_type)))?;
    Ok(AddressResponse { address: address })
}

pub fn query_terra_address(deps: Deps, env: Env, node: Vec<u8>) -> StdResult<AddressResponse> {
    return query_address(deps, env, node, LUNA_COIN_TYPE);
}

pub fn set_text_data(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    node: Vec<u8>,
    key: String,
    value: String,
) -> Result<Response, ContractError> {
    only_authorized(&deps, &info, &node)?;
    TEXT_DATA.save(deps.storage, (node, key), &value)?;
    Ok(Response::default())
}

pub fn query_text_data(
    deps: Deps,
    _env: Env,
    node: Vec<u8>,
    key: String,
) -> StdResult<TextDataResponse> {
    let value = TEXT_DATA.load(deps.storage, (node, key))?;
    Ok(TextDataResponse {
        data: value.to_string(),
    })
}

pub fn set_content_hash(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    node: Vec<u8>,
    hash: Vec<u8>,
) -> Result<Response, ContractError> {
    only_authorized(&deps, &info, &node)?;
    CONTENT_HASH.save(deps.storage, node, &hash)?;
    Ok(Response::default())
}

pub fn query_content_hash(deps: Deps, _env: Env, node: Vec<u8>) -> StdResult<ContentHashResponse> {
    let value = CONTENT_HASH.load(deps.storage, node)?;
    Ok(ContentHashResponse { hash: value })
}

pub fn set_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    interface_id: u64,
    registry_address: String,
    owner: String,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info)?;
    let mut config = CONFIG.load(deps.storage)?;

    let registry_address = deps.api.addr_canonicalize(registry_address.as_str())?;
    let owner = deps.api.addr_canonicalize(owner.as_str())?;

    config.interface_id = interface_id;
    config.registry_address = registry_address.clone();
    config.owner = owner.clone();

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("method", "set_config")
        .add_attribute("interface_id", interface_id.to_string())
        .add_attribute("registry_address", registry_address.clone().to_string())
        .add_attribute("owner", owner.clone().to_string()))
}

pub fn get_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    let owner = deps.api.addr_humanize(&config.owner)?;
    let registry_address = deps.api.addr_humanize(&config.registry_address)?;
    Ok(ConfigResponse {
        interface_id: config.interface_id,
        registry_address,
        owner,
    })
}
