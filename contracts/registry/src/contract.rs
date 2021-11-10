use crate::error::ContractError;
use crate::handler::{
    get_config, is_node_owner, query_is_approved_for_all, query_record, query_record_by_node,
    set_approval_for_all, set_config, set_owner, set_record, set_resolver, set_subnode_owner,
    set_ttl,
};
use crate::state::{Config, Record, CONFIG, RECORDS};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use tns::registry::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let sender = deps.api.addr_canonicalize(info.sender.as_str())?;
    let temp_resolver = deps.api.addr_canonicalize(env.contract.address.as_str())?;
    CONFIG.save(
        deps.storage,
        &Config {
            default_resolver: temp_resolver, // This will be set as resolver address once deployed
            owner: sender.clone(),
        },
    )?;
    let resolver = deps.api.addr_canonicalize(env.contract.address.as_str())?;
    RECORDS.save(
        deps.storage,
        vec![0u8; 32],
        &Record {
            owner: sender,
            resolver,
            ttl: 0,
        },
    )?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetRecord {
            node,
            owner,
            resolver,
            ttl,
        } => set_record(deps, env, info, node, owner, resolver, ttl),
        ExecuteMsg::SetSubnodeOwner { node, label, owner } => {
            set_subnode_owner(deps, env, info, node, label, owner)
        }
        ExecuteMsg::SetOwner { node, owner } => set_owner(deps, env, info, node, owner),
        ExecuteMsg::SetResolver { node, resolver } => set_resolver(deps, env, info, node, resolver),
        ExecuteMsg::SetTTL { node, ttl } => set_ttl(deps, env, info, node, ttl),
        ExecuteMsg::SetApprovalForAll {
            node,
            operator,
            approved,
        } => set_approval_for_all(deps, env, info, node, operator, approved),
        ExecuteMsg::SetConfig {
            default_resolver,
            owner,
        } => set_config(deps, env, info, default_resolver, owner),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetRecord { name } => to_binary(&query_record(deps, env, name)?),
        QueryMsg::GetRecordByNode { node } => to_binary(&query_record_by_node(deps, env, node)?),
        QueryMsg::GetIsNodeOwner { node, address } => {
            to_binary(&is_node_owner(deps, env, node, address)?)
        }
        QueryMsg::GetIsApprovedForAll { owner, operator } => {
            to_binary(&query_is_approved_for_all(deps, env, owner, operator)?)
        }
        QueryMsg::GetConfig {} => to_binary(&get_config(deps)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
