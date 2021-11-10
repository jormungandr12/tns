use crate::error::ContractError;
use crate::msg::{
    CommitmentTimestampResponse, GetCommitmentResponse, IsValidNameResponse,
    MaxCommitmentAgeResponse, MinCommitmentAgeResponse, MinRegistrationDurationResponse,
    NodeInfoResponse, NodehashResponse, OwnerResponse, PriceResponse, RegistrarResponse,
    RentPriceResponse, TokenIdResponse,
};
use crate::state::{COMMITMENTS, CONFIG, REGISTER_FEE_DENOM};
use cosmwasm_std::{
    to_binary, BalanceResponse, BankQuery, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    QueryRequest, Response, StdError, StdResult, Uint128, WasmMsg, WasmQuery,
};
use hex;
use terraswap::asset::{Asset, AssetInfo};
use tns::registrar::{
    ExecuteMsg as RegistrarExecuteMsg, Extension, GetBaseNodeResponse, GetRegistryResponse,
    IsAvailableResponse, QueryMsg as RegistrarQueryMsg,
};
use tns::registry::ExecuteMsg as RegistryExecuteMsg;
use tns::resolver::ExecuteMsg as ResolverExecuteMsg;
use tns::utils::{get_label_from_name, get_token_id_from_label, keccak256};
use unicode_segmentation::UnicodeSegmentation;

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

pub fn withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info)?;

    let balance_response: BalanceResponse =
        deps.querier.query(&QueryRequest::Bank(BankQuery::Balance {
            address: env.contract.address.to_string(),
            denom: String::from(REGISTER_FEE_DENOM),
        }))?;
    let amount = balance_response.amount.amount;
    let total_asset = Asset {
        info: AssetInfo::NativeToken {
            denom: balance_response.amount.denom,
        },
        amount,
    };
    let message = total_asset.into_msg(&deps.querier, info.sender);
    Ok(Response::new()
        .add_message(message?)
        .add_attribute("method", "withdraw")
        .add_attribute("amount", amount))
}

pub fn set_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    max_commitment_age: u64,
    min_commitment_age: u64,
    min_registration_duration: u64,
    tier1_price: u64,
    tier2_price: u64,
    tier3_price: u64,
    registrar_address: String,
    owner: String,
    enable_registration: bool,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info)?;
    let mut config = CONFIG.load(deps.storage)?;

    let registrar_address = deps.api.addr_canonicalize(registrar_address.as_str())?;
    let owner = deps.api.addr_canonicalize(owner.as_str())?;

    config.max_commitment_age = max_commitment_age;
    config.min_commitment_age = min_commitment_age;
    config.min_registration_duration = min_registration_duration;
    config.tier1_price = tier1_price;
    config.tier2_price = tier2_price;
    config.tier3_price = tier3_price;
    config.registrar_address = registrar_address.clone();
    config.owner = owner.clone();
    config.enable_registration = enable_registration;

    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("method", "set_config")
        .add_attribute("max_commitment_age", max_commitment_age.to_string())
        .add_attribute("min_commitment_age", min_commitment_age.to_string())
        .add_attribute(
            "min_registration_duration",
            min_registration_duration.to_string(),
        )
        .add_attribute("tier1_price", tier1_price.to_string())
        .add_attribute("tier2_price", tier2_price.to_string())
        .add_attribute("tier3_price", tier3_price.to_string())
        .add_attribute("registrar_address", registrar_address.clone().to_string())
        .add_attribute(
            "enable_registration",
            enable_registration.clone().to_string(),
        )
        .add_attribute("owner", owner.clone().to_string()))
}

pub fn set_enable_registration(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    enable_registration: bool,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info)?;
    let mut config = CONFIG.load(deps.storage)?;
    config.enable_registration = enable_registration;
    CONFIG.save(deps.storage, &config)?;
    Ok(Response::new()
        .add_attribute("method", "set_enable_registration")
        .add_attribute("enable_registration", enable_registration.to_string()))
}

pub fn commit(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    commitment: String,
) -> Result<Response, ContractError> {
    validate_enable_registration(deps.as_ref())?;

    let config = CONFIG.load(deps.storage)?;

    let last_commit_time = COMMITMENTS
        .may_load(deps.storage, commitment.clone())?
        .unwrap_or(0);
    let current = env.block.time.seconds();

    if last_commit_time + config.max_commitment_age > current {
        return Err(ContractError::RecommitTooEarly {
            commit_expired: last_commit_time + config.max_commitment_age,
            current,
        });
    }

    COMMITMENTS.save(deps.storage, commitment.clone(), &current)?;

    Ok(Response::new()
        .add_attribute("method", "commit")
        .add_attribute("commitment", commitment))
}

fn validate_name(deps: Deps, name: String) -> Result<(), ContractError> {
    if !get_is_valid_name(&name)?.is_valid_name {
        return Err(ContractError::InvalidName {});
    }

    if !is_available_name(deps, &name)? || !get_is_valid_name(&name)?.is_valid_name {
        return Err(ContractError::UnavailabledName {});
    }
    Ok(())
}

pub fn consume_commitment(
    deps: DepsMut,
    env: Env,
    commitment: String,
) -> Result<(), ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let commit_time = COMMITMENTS.may_load(deps.storage, commitment.clone())?;
    if commit_time.is_none() {
        return Err(ContractError::ConsumeNonexistCommitment { commitment });
    }

    let commit_time = commit_time.unwrap();
    let current = env.block.time.seconds();
    if commit_time + config.min_commitment_age > current
        || commit_time + config.max_commitment_age < current
    {
        return Err(ContractError::CommitmentIsTooEarlyOrExpired {
            commit_expired: commit_time + config.max_commitment_age,
            commit_matured: commit_time + config.min_commitment_age,
            current,
        });
    }

    COMMITMENTS.remove(deps.storage, commitment);
    Ok(())
}

pub fn get_cost(deps: Deps, name: String, duration: u64) -> Result<Uint128, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let min_duration = config.min_registration_duration;
    let name_length = name.graphemes(true).count();
    if name_length < 3 {
        return Err(ContractError::NameTooShort {});
    }
    if duration < min_duration {
        return Err(ContractError::DurationTooShort {
            input_duration: duration,
            min_duration: min_duration,
        });
    }

    let base_cost = match name_length {
        3 => config.tier1_price,
        4 => config.tier2_price,
        _ => config.tier3_price,
    };
    Ok(Uint128::from(base_cost).multiply_ratio(duration, 31_536_000u64))
}

pub fn get_price(deps: Deps) -> StdResult<PriceResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(PriceResponse {
        tier1_price: config.tier1_price,
        tier2_price: config.tier2_price,
        tier3_price: config.tier3_price,
    })
}

fn _register(
    deps: DepsMut,
    env: Env,
    name: String,
    owner: String,
    duration: u64,
    resolver: Option<String>,
    address: Option<String>,
) -> Result<Vec<CosmosMsg>, ContractError> {
    let mut messages: Vec<CosmosMsg> = vec![];

    let config = CONFIG.load(deps.storage)?;
    let registrar_address = deps
        .api
        .addr_humanize(&config.registrar_address)?
        .to_string();

    let label: Vec<u8> = get_label_from_name(&name);
    let token_id = get_token_id_from_label(&label);

    // Register this contract to be temporary owner of the node at registrar
    let register_registrar_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: registrar_address.clone(),
        msg: to_binary(&RegistrarExecuteMsg::<Extension>::Register {
            id: token_id.clone(),
            owner: env.contract.address.to_string(),
            name: name.clone(),
            duration,
        })?,
        funds: vec![],
    });
    messages.push(register_registrar_msg);

    let get_registry_response: GetRegistryResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: registrar_address.clone(),
            msg: to_binary(&RegistrarQueryMsg::GetRegistry {})?,
        }))?;
    let registry_address = String::from(get_registry_response.registry);

    // Set resolver of the node at registry
    let nodehash = get_nodehash(deps.as_ref(), label)?;
    let registry_set_resolver_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: registry_address.clone(),
        msg: to_binary(&RegistryExecuteMsg::SetResolver {
            node: nodehash.clone(),
            resolver: resolver.clone(),
        })?,
        funds: vec![],
    });
    messages.push(registry_set_resolver_msg);

    // Set address at resolver
    if let Some(address) = address {
        let set_address_resolver_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: resolver.unwrap_or(registry_address),
            msg: to_binary(&ResolverExecuteMsg::SetTerraAddress {
                node: nodehash,
                address: address,
            })?,
            funds: vec![],
        });
        messages.push(set_address_resolver_msg);
    }

    // Transfer ownership of the node to user
    let reclaim_registrar_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: registrar_address.clone(),
        msg: to_binary(&RegistrarExecuteMsg::<Extension>::Reclaim {
            id: token_id.clone(),
            owner: owner.clone(),
        })?,
        funds: vec![],
    });
    messages.push(reclaim_registrar_msg);

    // Transfer ownership of NFT to user
    let transfer_nft_registrar_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: registrar_address,
        msg: to_binary(&RegistrarExecuteMsg::<Extension>::TransferNft {
            recipient: owner,
            token_id: token_id,
        })?,
        funds: vec![],
    });
    messages.push(transfer_nft_registrar_msg);

    Ok(messages)
}

fn validate_register_fund(
    deps: Deps,
    _env: Env,
    info: MessageInfo,
    name: String,
    duration: u64,
) -> Result<(), ContractError> {
    let cost: Uint128 = get_cost(deps, name.clone(), duration)?;
    let base_fund = &Coin {
        denom: String::from(REGISTER_FEE_DENOM),
        amount: Uint128::from(0u128),
    };
    let fund = info
        .funds
        .iter()
        .find(|fund| fund.denom == String::from(REGISTER_FEE_DENOM))
        .unwrap_or(base_fund);
    if fund.amount < cost {
        return Err(ContractError::InsufficientFund {
            amount: fund.amount,
            required: cost,
        });
    }

    Ok(())
}

fn validate_enable_registration(deps: Deps) -> Result<(), ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if !config.enable_registration {
        return Err(ContractError::RegistrationDisabled {});
    }
    Ok(())
}

pub fn register(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    name: String,
    owner: String,
    duration: u64,
    secret: String,
    resolver: Option<String>,
    address: Option<String>,
) -> Result<Response, ContractError> {
    validate_name(deps.as_ref(), name.clone())?;
    validate_enable_registration(deps.as_ref())?;

    let commitment_response = get_commitment(&name, &owner, &secret, &resolver, &address)?;
    let commitment = commitment_response.commitment;
    consume_commitment(deps.branch(), env.clone(), commitment)?;

    validate_register_fund(
        deps.as_ref(),
        env.clone(),
        info,
        name.clone(),
        duration.clone(),
    )?;

    let messages = _register(
        deps.branch(),
        env.clone(),
        name.clone(),
        owner,
        duration,
        resolver,
        address,
    )?;

    let label: Vec<u8> = get_label_from_name(&name);
    let token_id = get_token_id_from_label(&label);
    let nodehash = get_nodehash(deps.as_ref(), label.clone())?;

    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("method", "register")
        .add_attribute("name", name)
        .add_attribute("label", format!("{:?}", label.clone()))
        .add_attribute("token_id", token_id)
        .add_attribute("nodehash", format!("{:?}", nodehash)))
}

pub fn owner_register(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    name: String,
    owner: String,
    duration: u64,
    resolver: Option<String>,
    address: Option<String>,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info)?;

    if !is_available_name(deps.as_ref(), &name)? {
        return Err(ContractError::UnavailabledName {});
    }

    let messages = _register(
        deps.branch(),
        env.clone(),
        name.clone(),
        owner,
        duration,
        resolver,
        address,
    )?;

    let label: Vec<u8> = get_label_from_name(&name);
    let token_id = get_token_id_from_label(&label);
    let nodehash = get_nodehash(deps.as_ref(), label.clone())?;

    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("method", "owner_register")
        .add_attribute("name", name)
        .add_attribute("label", format!("{:?}", label.clone()))
        .add_attribute("token_id", token_id)
        .add_attribute("nodehash", format!("{:?}", nodehash)))
}

fn _renew(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    token_id: String,
    duration: u64,
) -> Result<Vec<CosmosMsg>, ContractError> {
    let mut messages: Vec<CosmosMsg> = vec![];
    let config = CONFIG.load(deps.storage)?;
    let registrar_address = deps
        .api
        .addr_humanize(&config.registrar_address)?
        .to_string();

    let renew_registrar_message: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: registrar_address.clone(),
        msg: to_binary(&RegistrarExecuteMsg::<Extension>::Renew {
            id: token_id.clone(),
            duration,
        })?,
        funds: vec![],
    });
    messages.push(renew_registrar_message);

    Ok(messages)
}

pub fn owner_renew(
    mut deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    name: String,
    duration: u64,
) -> Result<Response, ContractError> {
    only_owner(deps.as_ref(), &info)?;
    let label = get_label_from_name(&name);
    let token_id = get_token_id_from_label(&label);
    let nodehash = get_nodehash(deps.as_ref(), label.clone())?;
    let messages = _renew(deps.branch(), _env, info, token_id.clone(), duration)?;
    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("method", "owner_renew")
        .add_attribute("name", name)
        .add_attribute("duration", duration.to_string())
        .add_attribute("label", format!("{:?}", label))
        .add_attribute("token_id", token_id)
        .add_attribute("nodehash", format!("{:?}", nodehash)))
}

pub fn renew(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    name: String,
    duration: u64,
) -> Result<Response, ContractError> {
    validate_register_fund(
        deps.as_ref(),
        env.clone(),
        info.clone(),
        name.clone(),
        duration,
    )?;
    let label = get_label_from_name(&name);
    let token_id = get_token_id_from_label(&label);
    let nodehash = get_nodehash(deps.as_ref(), label.clone())?;
    let messages = _renew(deps.branch(), env, info, token_id.clone(), duration)?;
    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("method", "renew")
        .add_attribute("name", name)
        .add_attribute("duration", duration.to_string())
        .add_attribute("label", format!("{:?}", label.clone()))
        .add_attribute("token_id", token_id)
        .add_attribute("nodehash", format!("{:?}", nodehash)))
}

pub fn get_commitment(
    name: &String,
    owner: &String,
    secret: &String,
    resolver: &Option<String>,
    address: &Option<String>,
) -> StdResult<GetCommitmentResponse> {
    let label = get_label_from_name(name);

    let arr = [
        &label[..],
        owner.as_bytes(),
        resolver.as_deref().unwrap_or(&String::from("")).as_bytes(),
        address.as_deref().unwrap_or(&String::from("")).as_bytes(),
        secret.as_bytes(),
    ]
    .concat();

    let commitment_vec = keccak256(&arr);
    Ok(GetCommitmentResponse {
        commitment: hex::encode(commitment_vec),
    })
}

pub fn get_nodehash(deps: Deps, label: Vec<u8>) -> StdResult<Vec<u8>> {
    let config = CONFIG.load(deps.storage)?;
    let registrar_address = deps
        .api
        .addr_humanize(&config.registrar_address)?
        .to_string();

    let get_base_node_response: GetBaseNodeResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: registrar_address.clone(),
            msg: to_binary(&RegistrarQueryMsg::GetBaseNode {})?,
        }))?;
    let base_node = get_base_node_response.base_node;

    let arr = [&hex::decode(base_node).unwrap(), &label[..]].concat();

    let nodehash = keccak256(&arr);
    Ok(nodehash)
}

pub fn is_available_name(deps: Deps, name: &String) -> StdResult<bool> {
    let label = get_label_from_name(name);
    let id = get_token_id_from_label(&label);
    let config = CONFIG.load(deps.storage)?;
    let registrar_address = deps
        .api
        .addr_humanize(&config.registrar_address)?
        .to_string();
    let is_available_response: IsAvailableResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: registrar_address,
            msg: to_binary(&RegistrarQueryMsg::IsAvailable { id })?,
        }))?;
    return Ok(is_available_response.available);
}

pub fn get_max_commitment_age(deps: Deps) -> StdResult<MaxCommitmentAgeResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(MaxCommitmentAgeResponse {
        age: config.max_commitment_age,
    })
}

pub fn get_min_commitment_age(deps: Deps) -> StdResult<MinCommitmentAgeResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(MinCommitmentAgeResponse {
        age: config.min_commitment_age,
    })
}

pub fn get_owner(deps: Deps) -> StdResult<OwnerResponse> {
    let config = CONFIG.load(deps.storage)?;
    let owner = deps.api.addr_humanize(&config.owner)?;
    Ok(OwnerResponse { owner })
}

pub fn get_registrar(deps: Deps) -> StdResult<RegistrarResponse> {
    let config = CONFIG.load(deps.storage)?;
    let registrar_address = deps.api.addr_humanize(&config.registrar_address)?;
    Ok(RegistrarResponse { registrar_address })
}

pub fn get_rent_price(deps: Deps, name: String, duration: u64) -> StdResult<RentPriceResponse> {
    let cost = get_cost(deps, name, duration);
    if let Err(_err) = cost {
        return Err(StdError::generic_err("error"));
    }
    Ok(RentPriceResponse {
        price: cost.unwrap(),
    })
}

pub fn get_commitment_timestamp(
    deps: Deps,
    commitment: String,
) -> StdResult<CommitmentTimestampResponse> {
    let timestamp = COMMITMENTS.load(deps.storage, commitment)?;
    Ok(CommitmentTimestampResponse { timestamp })
}

pub fn get_min_registration_duration(deps: Deps) -> StdResult<MinRegistrationDurationResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(MinRegistrationDurationResponse {
        duration: config.min_registration_duration,
    })
}

pub fn get_is_valid_name(name: &String) -> StdResult<IsValidNameResponse> {
    let graphemes = name.graphemes(true).collect::<Vec<&str>>();
    let name_length = graphemes.len();
    if graphemes[0usize] == "-" {
        return Ok(IsValidNameResponse {
            is_valid_name: false,
        });
    }
    let is_valid_name = name_length >= 3
        && name.chars().all(|c| -> bool {
            match c {
                '0'..='9' => true,
                'a'..='z' => true,
                '-' => true,
                _c => false,
            }
        });
    Ok(IsValidNameResponse { is_valid_name })
}

pub fn get_node_info_from_name(deps: Deps, name: &String) -> StdResult<NodeInfoResponse> {
    let label: Vec<u8> = get_label_from_name(&name);
    let token_id = get_token_id_from_label(&label);
    let node = get_nodehash(deps, label.clone())?;
    Ok(NodeInfoResponse {
        label,
        token_id,
        node,
    })
}

pub fn get_token_id_from_name(name: &String) -> StdResult<TokenIdResponse> {
    let label: Vec<u8> = get_label_from_name(&name);
    let token_id = get_token_id_from_label(&label);
    Ok(TokenIdResponse { token_id })
}

pub fn get_nodehash_from_name(deps: Deps, name: &String) -> StdResult<NodehashResponse> {
    let label: Vec<u8> = get_label_from_name(&name);
    let node = get_nodehash(deps, label)?;
    Ok(NodehashResponse { node })
}
