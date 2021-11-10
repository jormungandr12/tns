use crate::error::ContractError;
use crate::state::{Cw721Contract, CONFIG, CONTROLLERS, EXPIRIES};
use crate::utils::decode_node_string_to_bytes;
use cosmwasm_std::{to_binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, WasmMsg};
use cw721::CustomMsg;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tns::registry::ExecuteMsg as RegistryExecuteMsg;
use tns::utils::{generate_image, get_label_from_name, get_token_id_from_label};

fn only_owner(deps: Deps, info: MessageInfo) -> Result<bool, ContractError> {
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

fn only_controller(deps: Deps, info: &MessageInfo) -> Result<bool, ContractError> {
    let is_controller = CONTROLLERS
        .may_load(deps.storage, info.sender.clone())?
        .unwrap_or(false);
    if !is_controller {
        return Err(ContractError::NotController {
            sender: info.sender.to_string()
        });
    }
    Ok(is_controller)
}

fn validate_id(id: String, name: String) -> Result<bool, ContractError> {
    let label = get_label_from_name(&name);
    let token_id = get_token_id_from_label(&label);
    if id != token_id {
        return Err(ContractError::IdAndNameNotMatch {});
    }
    Ok(true)
}

impl<'a, T, C> Cw721Contract<'a, T, C>
where
    T: Serialize + DeserializeOwned + Clone + Default,
    C: CustomMsg,
{
    pub fn register(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        id: String,
        owner: String,
        name: String,
        duration: u64,
    ) -> Result<Response<C>, ContractError> {
        let mut messages: Vec<CosmosMsg<C>> = vec![];
        only_controller(deps.as_ref(), &info)?;
        validate_id(id.clone(), name.clone())?;
        if !self
            .is_available(deps.as_ref(), &env, id.clone())?
            .available
        {
            return Err(ContractError::NotAvailable {});
        }

        let config = CONFIG.load(deps.storage)?;
        let registry_address = deps.api.addr_humanize(&config.registry_address)?;

        let expire = env.block.time.seconds() + duration;
        EXPIRIES.save(deps.storage, id.clone(), &expire)?;
        let token = self.tokens.may_load(deps.storage, &id)?;
        if let Some(_token) = token {
            let token_id = id.clone();
            self.decrease_tokens(deps.storage)?;
            self.tokens.remove(deps.storage, &token_id)?;
        }

        let mint_response = self._mint(
            deps,
            env.clone(),
            info,
            owner.clone(),
            name.clone() + "." + &config.base_name,
            None,
            Some(generate_image(
                name.clone() + "." + &config.base_name,
                env.block.time.seconds(),
            )),
            T::default(),
            id.clone(),
        )?;

        let label = decode_node_string_to_bytes(id.clone()).unwrap();
        let set_subnode_owner_registry_msg: CosmosMsg<C> = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: registry_address.to_string(),
            msg: to_binary(&RegistryExecuteMsg::SetSubnodeOwner {
                node: config.base_node,
                owner: owner.clone(),
                label,
            })?,
            funds: vec![],
        });
        messages.push(set_subnode_owner_registry_msg);
        Ok(Response::<C>::new()
            .add_attributes(mint_response.attributes)
            .add_messages(messages)
            .add_attribute("method", "register")
            .add_attribute("id", id)
            .add_attribute("owner", owner)
            .add_attribute("duration", duration.to_string()))
    }

    pub fn renew(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        id: String,
        duration: u64,
    ) -> Result<Response<C>, ContractError> {
        only_controller(deps.as_ref(), &info)?;
        let config = CONFIG.load(deps.storage)?;
        let expiry = EXPIRIES.load(deps.storage, id.clone())?;
        if expiry + config.grace_period < env.block.time.seconds() {
            return Err(ContractError::Expired {});
        }
        let new_expiry = expiry + duration;
        EXPIRIES.save(deps.storage, id.clone(), &new_expiry)?;
        Ok(Response::new()
            .add_attribute("method", "renew")
            .add_attribute("id", id)
            .add_attribute("duration", duration.to_string()))
    }

    pub fn add_controller(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        address: String,
    ) -> Result<Response<C>, ContractError> {
        only_owner(deps.as_ref(), info)?;
        let controller_addr = deps.api.addr_validate(address.as_str())?;
        CONTROLLERS.save(deps.storage, controller_addr, &true)?;
        Ok(Response::new()
            .add_attribute("method", "add_controller")
            .add_attribute("controller", address))
    }

    pub fn remove_controller(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        address: String,
    ) -> Result<Response<C>, ContractError> {
        only_owner(deps.as_ref(), info)?;
        let controller_addr = deps.api.addr_validate(address.as_str())?;
        CONTROLLERS.save(deps.storage, controller_addr, &false)?;
        Ok(Response::new()
            .add_attribute("method", "add_controller")
            .add_attribute("controller", address))
    }

    pub fn set_config(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        grace_period: u64,
        registry_address: String,
        owner: String,
    ) -> Result<Response<C>, ContractError> {
        only_owner(deps.as_ref(), info)?;
        let mut config = CONFIG.load(deps.storage)?;

        let registry_address = deps.api.addr_canonicalize(registry_address.as_str())?;
        let owner = deps.api.addr_canonicalize(owner.as_str())?;

        config.grace_period = grace_period;
        config.registry_address = registry_address.clone();
        config.owner = owner.clone();

        CONFIG.save(deps.storage, &config)?;
        Ok(Response::new()
            .add_attribute("method", "set_config")
            .add_attribute("grace_period", grace_period.to_string())
            .add_attribute("registry_address", registry_address.clone().to_string())
            .add_attribute("owner", owner.clone().to_string()))
    }

    pub fn reclaim(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        id: String,
        owner: String,
    ) -> Result<Response<C>, ContractError> {
        let token = self.tokens.load(deps.storage, &id)?;
        self.check_can_send(deps.as_ref(), &env, &info, &token)?;

        let mut messages: Vec<CosmosMsg<C>> = vec![];
        let config = CONFIG.load(deps.storage)?;
        let registry_address = deps.api.addr_humanize(&config.registry_address)?;
        let set_subnode_owner_registry_msg: CosmosMsg<C> = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: registry_address.to_string(),
            msg: to_binary(&RegistryExecuteMsg::SetSubnodeOwner {
                node: config.base_node,
                label: hex::decode(id).unwrap(),
                owner,
            })?,
            funds: vec![],
        });
        messages.push(set_subnode_owner_registry_msg);

        Ok(Response::<C>::new().add_messages(messages))
    }
}
