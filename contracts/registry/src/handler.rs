use crate::error::ContractError;
use crate::state::{Record, OPERATORS, RECORDS, CONFIG};
use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use tns::registry::{OperatorResponse, RecordResponse, ConfigResponse};
use tns::utils::keccak256;
use tns::utils::namehash;

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

fn only_authorized(deps: &DepsMut, info: &MessageInfo, node: &Vec<u8>) -> Result<bool, ContractError> {
    let record_option = RECORDS.may_load(deps.storage, node.to_vec())?;
    let canonical_sender = deps.api.addr_canonicalize(info.sender.as_str())?;
    if let Some(record) = record_option {
        if record.owner == canonical_sender {
            return Ok(true)
        }

        let operator_option = OPERATORS.may_load(
            deps.storage,
            (record.owner.to_vec(), canonical_sender.to_vec()),
        )?;
        if let Some(operator) = operator_option {
            if operator {
                return Ok(true);
            }
        }
    }
    return Err(ContractError::NotNodeOwner {
        sender: info.sender.to_string(),
        node: format!("{:?}", node.clone()),
    });
}

pub fn set_subnode_owner(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    node: Vec<u8>,
    label: Vec<u8>,
    owner: String,
) -> Result<Response, ContractError> {
    only_authorized(&deps, &info, &node)?;
    let subnode = keccak256(&[node, label].concat());
    _set_owner(deps, env, subnode, owner)?;
    Ok(Response::new())
}

fn _set_owner(
    deps: DepsMut,
    _env: Env,
    node: Vec<u8>,
    owner: String,
) -> Result<Response, ContractError> {
    let record_option = RECORDS.may_load(deps.storage, node.clone())?;
    let canonical_owner = deps.api.addr_canonicalize(owner.as_str())?;
    if let Some(mut record) = record_option {
        record.owner = canonical_owner;
        RECORDS.save(deps.storage, node.clone(), &record)?;
        return Ok(Response::default());
    }

    let config = CONFIG.load(deps.storage)?;

    RECORDS.save(
        deps.storage,
        node,
        &Record {
            owner: canonical_owner,
            resolver: config.default_resolver,
            ttl: 0,
        },
    )?;
    return Ok(Response::default());
}

pub fn set_record(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    node: Vec<u8>,
    owner: String,
    resolver: Option<String>,
    ttl: u64,
) -> Result<Response, ContractError> {
    only_authorized(&deps, &info, &node)?;
    let owner = deps.api.addr_canonicalize(owner.as_str())?;
    let config = CONFIG.load(deps.storage)?;
    let default_resolver = deps.api.addr_humanize(&config.default_resolver)?;
    let canonical_resolver = deps.api.addr_canonicalize(
        resolver.unwrap_or(default_resolver.to_string()).as_str(),
    )?;
    RECORDS.save(
        deps.storage,
        node,
        &Record {
            owner,
            resolver: canonical_resolver,
            ttl,
        },
    )?;
    Ok(Response::default())
}

pub fn set_owner(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    node: Vec<u8>,
    owner: String,
) -> Result<Response, ContractError> {
    only_authorized(&deps, &info, &node)?;
    _set_owner(deps, env, node, owner)?;
    Ok(Response::default())
}

pub fn set_ttl(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    node: Vec<u8>,
    ttl: u64,
) -> Result<Response, ContractError> {
    only_authorized(&deps, &info, &node)?;
    let mut record = RECORDS.load(deps.storage, node.clone())?;
    record.ttl = ttl;
    RECORDS.save(deps.storage, node.clone(), &record)?;
    Ok(Response::default())
}

pub fn set_resolver(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    node: Vec<u8>,
    resolver: Option<String>,
) -> Result<Response, ContractError> {
    only_authorized(&deps, &info, &node)?;
    let mut record = RECORDS.load(deps.storage, node.clone())?;
    let config = CONFIG.load(deps.storage)?;
    let default_resolver = deps.api.addr_humanize(&config.default_resolver)?;
    let canonical_resolver = deps.api.addr_canonicalize(
        resolver.unwrap_or(default_resolver.to_string()).as_str(),
    )?;
    record.resolver = canonical_resolver;
    RECORDS.save(deps.storage, node.clone(), &record)?;
    Ok(Response::default())
}

pub fn is_node_owner(deps: Deps, _env: Env, node: Vec<u8>, address: String) -> StdResult<bool> {
    let record_option = RECORDS.may_load(deps.storage, node.to_vec())?;
    let canonical_sender = deps.api.addr_canonicalize(&address)?;
    if let Some(record) = record_option {
        if record.owner == canonical_sender {
            return Ok(true);
        }

        let operator_option = OPERATORS.may_load(
            deps.storage,
            (record.owner.to_vec(), canonical_sender.to_vec()),
        )?;
        if let Some(operator) = operator_option {
            return Ok(operator);
        }
    }
    return Ok(false);
}

pub fn query_record_by_node(deps: Deps, _env: Env, node: Vec<u8>) -> StdResult<RecordResponse> {
    let record = RECORDS.load(deps.storage, node)?;
    let owner = deps.api.addr_humanize(&record.owner)?;
    let resolver = deps.api.addr_humanize(&record.resolver)?;
    let ttl = record.ttl;
    Ok(RecordResponse {
        owner,
        resolver,
        ttl,
    })
}

pub fn query_record(deps: Deps, _env: Env, name: String) -> StdResult<RecordResponse> {
    let node = namehash(name.as_str());
    let record = RECORDS.load(deps.storage, node)?;
    let owner = deps.api.addr_humanize(&record.owner)?;
    let resolver = deps.api.addr_humanize(&record.resolver)?;
    let ttl = record.ttl;
    Ok(RecordResponse {
        owner,
        resolver,
        ttl,
    })
}

pub fn set_approval_for_all(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    node: Vec<u8>,
    operator: String,
    approved: bool,
) -> Result<Response, ContractError> {
    only_authorized(&deps, &info, &node)?;
    let sender_address = deps.api.addr_canonicalize(info.sender.as_str())?;
    let operator_address = deps.api.addr_canonicalize(operator.as_str())?;
    OPERATORS.save(
        deps.storage,
        (sender_address.to_vec(), operator_address.to_vec()),
        &approved,
    )?;

    Ok(Response::default())
}

pub fn query_is_approved_for_all(
    deps: Deps,
    _env: Env,
    owner: String,
    operator: String,
) -> StdResult<OperatorResponse> {
    let owner_address = deps.api.addr_canonicalize(owner.as_str())?;
    let operator_address = deps.api.addr_canonicalize(operator.as_str())?;

    let value = OPERATORS.may_load(
        deps.storage,
        (owner_address.to_vec(), operator_address.to_vec()),
    )?;
    if let Some(record) = value {
        if record {
            return Ok(OperatorResponse { is_approve: true });
        }
    }

    return Ok(OperatorResponse { is_approve: false });
}

pub fn set_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    default_resolver: String,
    owner: String,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info)?;
    let mut config = CONFIG.load(deps.storage)?;

    let default_resolver = deps.api.addr_canonicalize(default_resolver.as_str())?;
    let owner = deps.api.addr_canonicalize(owner.as_str())?;

    config.default_resolver = default_resolver.clone();
    config.owner = owner.clone();

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("method", "set_config")
        .add_attribute("default_resolver", default_resolver.clone().to_string())
        .add_attribute("owner", owner.to_string())
    )
}

pub fn get_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    let owner = deps.api.addr_humanize(&config.owner)?;
    let default_resolver = deps.api.addr_humanize(&config.default_resolver)?;
    Ok(ConfigResponse {
        default_resolver,
        owner,
    })
}
