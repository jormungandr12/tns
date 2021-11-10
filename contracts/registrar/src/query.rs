use crate::state::{Approval, Cw721Contract, TokenInfo};
use crate::state::{CONFIG, EXPIRIES};
use crate::utils::encode_node_bytes_to_string;
use cosmwasm_std::{to_binary, Binary, BlockInfo, Deps, Env, Order, Pair, StdError, StdResult};
use cw0::maybe_addr;
use cw721::{
    AllNftInfoResponse, ApprovedForAllResponse, ContractInfoResponse, CustomMsg, Cw721Query,
    Expiration, NftInfoResponse, NumTokensResponse, OwnerOfResponse, TokensResponse
};
use cw_storage_plus::Bound;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tns::registrar::{
    GetBaseNodeResponse, GetExpiresResponse, GetRegistryResponse, IsAvailableResponse,
    GetGracePeriodResponse, ConfigResponse
};
use tns::registrar::{MinterResponse, QueryMsg};

const DEFAULT_LIMIT: u32 = 10;
const MAX_LIMIT: u32 = 30;

impl<'a, T, C> Cw721Query<T> for Cw721Contract<'a, T, C>
where
    T: Serialize + DeserializeOwned + Clone,
    C: CustomMsg,
{
    fn contract_info(&self, deps: Deps) -> StdResult<ContractInfoResponse> {
        self.contract_info.load(deps.storage)
    }

    fn num_tokens(&self, deps: Deps) -> StdResult<NumTokensResponse> {
        let count = self.token_count(deps.storage)?;
        Ok(NumTokensResponse { count })
    }

    fn nft_info(&self, deps: Deps, token_id: String) -> StdResult<NftInfoResponse<T>> {
        let info = self.tokens.load(deps.storage, &token_id)?;
        Ok(NftInfoResponse {
            name: info.name,
            description: info.description,
            image: info.image,
            extension: info.extension,
        })
    }

    fn owner_of(
        &self,
        deps: Deps,
        env: Env,
        token_id: String,
        include_expired: bool,
    ) -> StdResult<OwnerOfResponse> {
        let info = self.tokens.load(deps.storage, &token_id)?;
        Ok(OwnerOfResponse {
            owner: info.owner.to_string(),
            approvals: humanize_approvals(&env.block, &info, include_expired),
        })
    }

    fn all_approvals(
        &self,
        deps: Deps,
        env: Env,
        owner: String,
        include_expired: bool,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<ApprovedForAllResponse> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let start_addr = maybe_addr(deps.api, start_after)?;
        let start = start_addr.map(|addr| Bound::exclusive(addr.as_ref()));

        let owner_addr = deps.api.addr_validate(&owner)?;
        let res: StdResult<Vec<_>> = self
            .operators
            .prefix(&owner_addr)
            .range(deps.storage, start, None, Order::Ascending)
            .filter(|r| {
                include_expired || r.is_err() || !r.as_ref().unwrap().1.is_expired(&env.block)
            })
            .take(limit)
            .map(parse_approval)
            .collect();
        Ok(ApprovedForAllResponse { operators: res? })
    }

    fn tokens(
        &self,
        deps: Deps,
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<TokensResponse> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let start = start_after.map(Bound::exclusive);

        let owner_addr = deps.api.addr_validate(&owner)?;
        let pks: Vec<_> = self
            .tokens
            .idx
            .owner
            .prefix(owner_addr)
            .keys(deps.storage, start, None, Order::Ascending)
            .take(limit)
            .collect();

        let res: Result<Vec<_>, _> = pks.iter().map(|v| String::from_utf8(v.to_vec())).collect();
        let tokens = res.map_err(StdError::invalid_utf8)?;
        Ok(TokensResponse { tokens })
    }

    fn all_tokens(
        &self,
        deps: Deps,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<TokensResponse> {
        let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
        let start = start_after.map(|s| Bound::exclusive(s));

        let tokens: StdResult<Vec<String>> = self
            .tokens
            .range(deps.storage, start, None, Order::Ascending)
            .take(limit)
            .map(|item| item.map(|(k, _)| String::from_utf8_lossy(&k).to_string()))
            .collect();
        Ok(TokensResponse { tokens: tokens? })
    }

    fn all_nft_info(
        &self,
        deps: Deps,
        env: Env,
        token_id: String,
        include_expired: bool,
    ) -> StdResult<AllNftInfoResponse<T>> {
        let info = self.tokens.load(deps.storage, &token_id)?;
        Ok(AllNftInfoResponse {
            access: OwnerOfResponse {
                owner: info.owner.to_string(),
                approvals: humanize_approvals(&env.block, &info, include_expired),
            },
            info: NftInfoResponse {
                name: info.name,
                description: info.description,
                image: info.image,
                extension: info.extension,
            },
        })
    }
}

impl<'a, T, C> Cw721Contract<'a, T, C>
where
    T: Serialize + DeserializeOwned + Clone,
    C: CustomMsg,
{
    pub fn minter(&self, deps: Deps) -> StdResult<MinterResponse> {
        let minter_addr = self.minter.load(deps.storage)?;
        Ok(MinterResponse {
            minter: minter_addr.to_string(),
        })
    }

    pub fn is_available(
        &self,
        deps: Deps,
        env: &Env,
        id: String,
    ) -> StdResult<IsAvailableResponse> {
        let config = CONFIG.load(deps.storage)?;
        let expiry = EXPIRIES.may_load(deps.storage, id)?.unwrap_or(0);
        let available = expiry + config.grace_period < env.block.time.seconds();
        Ok(IsAvailableResponse { available })
    }
    pub fn get_expires(&self, deps: Deps, id: String) -> StdResult<GetExpiresResponse> {
        let expires = EXPIRIES.may_load(deps.storage, id)?.unwrap_or(0);
        Ok(GetExpiresResponse { expires })
    }

    pub fn get_base_node(&self, deps: Deps) -> StdResult<GetBaseNodeResponse> {
        let base_node = CONFIG.load(deps.storage)?.base_node;
        Ok(GetBaseNodeResponse {
            base_node: encode_node_bytes_to_string(base_node),
        })
    }

    pub fn get_registry(&self, deps: Deps) -> StdResult<GetRegistryResponse> {
        let registry_address = CONFIG.load(deps.storage)?.registry_address;
        let registry = deps.api.addr_humanize(&registry_address)?;
        Ok(GetRegistryResponse { registry })
    }

    pub fn get_grace_period(&self, deps: Deps) -> StdResult<GetGracePeriodResponse> {
        let grace_period = CONFIG.load(deps.storage)?.grace_period;
        Ok(GetGracePeriodResponse { grace_period })
    }

    pub fn get_config(&self, deps: Deps) -> StdResult<ConfigResponse> {
        let config = CONFIG.load(deps.storage)?;
        let owner = deps.api.addr_humanize(&config.owner)?;
        let registry_address = deps.api.addr_humanize(&config.registry_address)?;
        Ok(ConfigResponse {
            grace_period: config.grace_period,
            registry_address: registry_address,
            owner: owner,
            base_node: config.base_node,
            base_name: config.base_name,
        })
    }

    pub fn query(&self, deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        match msg {
            QueryMsg::IsAvailable { id } => to_binary(&self.is_available(deps, &env, id)?),
            QueryMsg::GetExpires { id } => to_binary(&self.get_expires(deps, id)?),
            QueryMsg::GetBaseNode {} => to_binary(&self.get_base_node(deps)?),
            QueryMsg::GetRegistry {} => to_binary(&self.get_registry(deps)?),
            QueryMsg::GetGracePeriod {} => to_binary(&self.get_grace_period(deps)?),
            QueryMsg::GetConfig {} => to_binary(&self.get_config(deps)?),

            QueryMsg::Minter {} => to_binary(&self.minter(deps)?),
            QueryMsg::ContractInfo {} => to_binary(&self.contract_info(deps)?),
            QueryMsg::NftInfo { token_id } => to_binary(&self.nft_info(deps, token_id)?),
            QueryMsg::OwnerOf {
                token_id,
                include_expired,
            } => {
                to_binary(&self.owner_of(deps, env, token_id, include_expired.unwrap_or(false))?)
            }
            QueryMsg::AllNftInfo {
                token_id,
                include_expired,
            } => to_binary(&self.all_nft_info(
                deps,
                env,
                token_id,
                include_expired.unwrap_or(false),
            )?),
            QueryMsg::ApprovedForAll {
                owner,
                include_expired,
                start_after,
                limit,
            } => to_binary(&self.all_approvals(
                deps,
                env,
                owner,
                include_expired.unwrap_or(false),
                start_after,
                limit,
            )?),
            QueryMsg::NumTokens {} => to_binary(&self.num_tokens(deps)?),
            QueryMsg::Tokens {
                owner,
                start_after,
                limit,
            } => to_binary(&self.tokens(deps, owner, start_after, limit)?),
            QueryMsg::AllTokens { start_after, limit } => {
                to_binary(&self.all_tokens(deps, start_after, limit)?)
            }
        }
    }
}

fn parse_approval(item: StdResult<Pair<Expiration>>) -> StdResult<cw721::Approval> {
    item.and_then(|(k, expires)| {
        let spender = String::from_utf8(k)?;
        Ok(cw721::Approval { spender, expires })
    })
}

fn humanize_approvals<T>(
    block: &BlockInfo,
    info: &TokenInfo<T>,
    include_expired: bool,
) -> Vec<cw721::Approval> {
    info.approvals
        .iter()
        .filter(|apr| include_expired || !apr.is_expired(block))
        .map(humanize_approval)
        .collect()
}

fn humanize_approval(approval: &Approval) -> cw721::Approval {
    cw721::Approval {
        spender: approval.spender.to_string(),
        expires: approval.expires,
    }
}
