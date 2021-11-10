use crate::error::ContractError;
use crate::handler::{
    get_config, query_address, query_content_hash, query_terra_address, query_text_data,
    set_address, set_config, set_content_hash, set_terra_address, set_text_data,
};
use crate::state::{Config, CONFIG};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use tns::resolver::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let registry_address = deps.api.addr_canonicalize(msg.registry_address.as_str())?;
    let sender = deps.api.addr_canonicalize(info.sender.as_str())?;
    CONFIG.save(
        deps.storage,
        &Config {
            interface_id: msg.interface_id,
            registry_address,
            owner: sender.clone(),
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
        ExecuteMsg::SetAddress {
            node,
            coin_type,
            address,
        } => set_address(deps, env, info, node, coin_type, address),
        ExecuteMsg::SetTerraAddress { node, address } => {
            set_terra_address(deps, env, info, node, address)
        }
        ExecuteMsg::SetTextData { node, key, value } => {
            set_text_data(deps, env, info, node, key, value)
        }
        ExecuteMsg::SetContentHash { node, hash } => set_content_hash(deps, env, info, node, hash),
        ExecuteMsg::SetConfig {
            interface_id,
            registry_address,
            owner,
        } => set_config(deps, env, info, interface_id, registry_address, owner),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetAddress { node, coin_type } => {
            to_binary(&query_address(deps, env, node, coin_type)?)
        }
        QueryMsg::GetTerraAddress { node } => to_binary(&query_terra_address(deps, env, node)?),
        QueryMsg::GetTextData { node, key } => to_binary(&query_text_data(deps, env, node, key)?),
        QueryMsg::GetContentHash { node } => to_binary(&query_content_hash(deps, env, node)?),
        QueryMsg::GetConfig {} => to_binary(&get_config(deps)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
