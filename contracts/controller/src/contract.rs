use crate::error::ContractError;
use crate::handler::{
    commit, get_commitment, get_commitment_timestamp, get_is_valid_name, get_max_commitment_age,
    get_min_commitment_age, get_min_registration_duration, get_node_info_from_name,
    get_nodehash_from_name, get_owner, get_price, get_registrar, get_rent_price,
    get_token_id_from_name, owner_register, owner_renew, register, renew, set_config,
    set_enable_registration, withdraw,
};
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, CONFIG};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let registrar_address = deps.api.addr_canonicalize(msg.registrar_address.as_str())?;
    let owner = deps.api.addr_canonicalize(info.sender.as_str())?;
    CONFIG.save(
        deps.storage,
        &Config {
            max_commitment_age: msg.max_commitment_age,
            min_commitment_age: msg.min_commitment_age,
            min_registration_duration: msg.min_registration_duration,
            tier1_price: msg.tier1_price,
            tier2_price: msg.tier2_price,
            tier3_price: msg.tier3_price,
            enable_registration: msg.enable_registration,
            registrar_address,
            owner,
        },
    )?;
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Commit { commitment } => commit(deps, env, info, commitment),
        ExecuteMsg::Register {
            name,
            owner,
            duration,
            secret,
            resolver,
            address,
        } => register(
            deps, env, info, name, owner, duration, secret, resolver, address,
        ),
        ExecuteMsg::Renew { name, duration } => renew(deps, env, info, name, duration),

        // Only owner
        ExecuteMsg::SetConfig {
            max_commitment_age,
            min_commitment_age,
            min_registration_duration,
            tier1_price,
            tier2_price,
            tier3_price,
            registrar_address,
            owner,
            enable_registration,
        } => set_config(
            deps,
            env,
            info,
            max_commitment_age,
            min_commitment_age,
            min_registration_duration,
            tier1_price,
            tier2_price,
            tier3_price,
            registrar_address,
            owner,
            enable_registration,
        ),
        ExecuteMsg::Withdraw {} => withdraw(deps, env, info),
        ExecuteMsg::OwnerRegister {
            name,
            owner,
            duration,
            resolver,
            address,
        } => owner_register(deps, env, info, name, owner, duration, resolver, address),
        ExecuteMsg::OwnerRenew { name, duration } => owner_renew(deps, env, info, name, duration),
        ExecuteMsg::SetEnableRegistration {
            enable_registration,
        } => set_enable_registration(deps, env, info, enable_registration),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCommitment {
            name,
            owner,
            secret,
            resolver,
            address,
        } => to_binary(&get_commitment(
            &name, &owner, &secret, &resolver, &address,
        )?),
        QueryMsg::CommitmentTimestamp { commitment } => {
            to_binary(&get_commitment_timestamp(deps, commitment)?)
        }
        QueryMsg::RentPrice { name, duration } => to_binary(&get_rent_price(deps, name, duration)?),

        QueryMsg::MaxCommitmentAge {} => to_binary(&get_max_commitment_age(deps)?),
        QueryMsg::MinCommitmentAge {} => to_binary(&get_min_commitment_age(deps)?),
        QueryMsg::MinRegistrationDuration {} => to_binary(&get_min_registration_duration(deps)?),
        QueryMsg::GetPrice {} => to_binary(&get_price(deps)?),
        QueryMsg::Registrar {} => to_binary(&get_registrar(deps)?),
        QueryMsg::Owner {} => to_binary(&get_owner(deps)?),

        QueryMsg::IsValidName { name } => to_binary(&get_is_valid_name(&name)?),
        QueryMsg::GetTokenId { name } => to_binary(&get_token_id_from_name(&name)?),
        QueryMsg::GetNodehash { name } => to_binary(&get_nodehash_from_name(deps, &name)?),
        QueryMsg::GetNodeInfo { name } => to_binary(&get_node_info_from_name(deps, &name)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}
