#![cfg(test)]
use crate::entry;
use crate::error::ContractError;
use crate::state::Cw721Contract;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{
    coins, from_binary, to_binary, Addr, CosmosMsg, DepsMut, Empty, Response, WasmMsg,
};
use cw0::Expiration;
use cw721;
use cw721::{
    Approval, ApprovedForAllResponse, ContractInfoResponse, Cw721Query, Cw721ReceiveMsg,
    NftInfoResponse, OwnerOfResponse,
};
use tns::registrar::{
    ConfigResponse, ExecuteMsg, Extension, InstantiateMsg, IsAvailableResponse, MintMsg, QueryMsg,
};
use tns::registry::ExecuteMsg as RegistryExecuteMsg;

const CONTRACT_NAME: &str = "Magic Power";
const SYMBOL: &str = "MGK";
const UST_BASE_NODE: &str = "749f2b479b45e5da8e4cbecd926ee9a6f78db5424fa6993b6ecababa5d736b12";
const BASE_NAME: &str = "ust";

fn setup_contract(deps: DepsMut<'_>) -> Cw721Contract<'static, Extension, Empty> {
    let contract = Cw721Contract::default();
    let msg = InstantiateMsg {
        name: CONTRACT_NAME.to_string(),
        base_name: BASE_NAME.to_string(),
        symbol: SYMBOL.to_string(),
        base_node: UST_BASE_NODE.to_string(),
        registry_address: String::from("registry_address"),
        grace_period: None,
    };
    let info = mock_info("creator", &[]);
    let res = contract.instantiate(deps, mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());
    contract
}

#[test]
fn proper_instantiation() {
    let mut deps = mock_dependencies(&[]);
    let contract = Cw721Contract::<Extension, Empty>::default();

    let msg = InstantiateMsg {
        name: CONTRACT_NAME.to_string(),
        symbol: SYMBOL.to_string(),
        base_node: UST_BASE_NODE.to_string(),
        base_name: BASE_NAME.to_string(),
        registry_address: String::from("hellooo"),
        grace_period: None,
    };
    let info = mock_info("creator", &[]);

    // we can just call .unwrap() to assert this was a success
    let res = contract
        .instantiate(deps.as_mut(), mock_env(), info, msg)
        .unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = contract.minter(deps.as_ref()).unwrap();
    assert_eq!("creator", res.minter);
    let info = contract.contract_info(deps.as_ref()).unwrap();
    assert_eq!(
        info,
        ContractInfoResponse {
            name: CONTRACT_NAME.to_string(),
            symbol: SYMBOL.to_string(),
        }
    );

    let count = contract.num_tokens(deps.as_ref()).unwrap();
    assert_eq!(0, count.count);

    // list the token_ids
    let tokens = contract.all_tokens(deps.as_ref(), None, None).unwrap();
    assert_eq!(0, tokens.tokens.len());
}

#[test]
fn minting() {
    let mut deps = mock_dependencies(&[]);
    let contract = setup_contract(deps.as_mut());

    let token_id = "petrify".to_string();
    let name = "Petrify with Gaze".to_string();
    let description = "Allows the owner to petrify anyone looking at him or her".to_string();

    let mint_msg = ExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id.clone(),
        owner: String::from("medusa"),
        name: name.clone(),
        description: Some(description.clone()),
        image: None,
        extension: Extension {},
    });

    // random cannot mint
    let random = mock_info("random", &[]);
    let err = contract
        .execute(deps.as_mut(), mock_env(), random, mint_msg.clone())
        .unwrap_err();
    assert_eq!(
        err,
        ContractError::Unauthorized {
            description: Some(String::from("sender is not minter")),
        }
    );

    // minter can mint
    let allowed = mock_info("creator", &[]);
    let _ = contract
        .execute(deps.as_mut(), mock_env(), allowed, mint_msg)
        .unwrap();

    // ensure num tokens increases
    let count = contract.num_tokens(deps.as_ref()).unwrap();
    assert_eq!(1, count.count);

    // unknown nft returns error
    let _ = contract
        .nft_info(deps.as_ref(), "unknown".to_string())
        .unwrap_err();

    // this nft info is correct
    let info = contract.nft_info(deps.as_ref(), token_id.clone()).unwrap();
    assert_eq!(
        info,
        NftInfoResponse::<Extension> {
            name,
            description,
            image: None,
            extension: Extension {},
        }
    );

    // owner info is correct
    let owner = contract
        .owner_of(deps.as_ref(), mock_env(), token_id.clone(), true)
        .unwrap();
    assert_eq!(
        owner,
        OwnerOfResponse {
            owner: String::from("medusa"),
            approvals: vec![],
        }
    );

    // Cannot mint same token_id again
    let mint_msg2 = ExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id.clone(),
        owner: String::from("hercules"),
        name: "copy cat".into(),
        description: None,
        image: None,
        extension: Extension {},
    });

    let allowed = mock_info("creator", &[]);
    let err = contract
        .execute(deps.as_mut(), mock_env(), allowed, mint_msg2)
        .unwrap_err();
    assert_eq!(err, ContractError::Claimed {});

    // list the token_ids
    let tokens = contract.all_tokens(deps.as_ref(), None, None).unwrap();
    assert_eq!(1, tokens.tokens.len());
    assert_eq!(vec![token_id], tokens.tokens);
}

#[test]
fn transferring_nft() {
    let mut deps = mock_dependencies(&[]);
    let contract = setup_contract(deps.as_mut());

    // Mint a token
    let token_id = "melt".to_string();
    let name = "Melting power".to_string();
    let description = "Allows the owner to melt anyone looking at him or her".to_string();

    let mint_msg = ExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id.clone(),
        owner: String::from("venus"),
        name,
        description: Some(description),
        image: None,
        extension: Extension {},
    });

    let minter = mock_info("creator", &[]);
    contract
        .execute(deps.as_mut(), mock_env(), minter, mint_msg)
        .unwrap();

    // random cannot transfer
    let random = mock_info("random", &[]);
    let transfer_msg = ExecuteMsg::TransferNft {
        recipient: String::from("random"),
        token_id: token_id.clone(),
    };

    let err = contract
        .execute(deps.as_mut(), mock_env(), random, transfer_msg)
        .unwrap_err();
    assert_eq!(
        err,
        ContractError::Unauthorized {
            description: Some(String::from("sender is neither owner nor operator")),
        }
    );

    // owner can
    let random = mock_info("venus", &[]);
    let transfer_msg = ExecuteMsg::TransferNft {
        recipient: String::from("random"),
        token_id: token_id.clone(),
    };

    let res = contract
        .execute(deps.as_mut(), mock_env(), random, transfer_msg)
        .unwrap();

    assert_eq!(
        res,
        Response::new()
            .add_attribute("action", "transfer_nft")
            .add_attribute("sender", "venus")
            .add_attribute("recipient", "random")
            .add_attribute("token_id", token_id)
    );
}

#[test]
fn sending_nft() {
    let mut deps = mock_dependencies(&[]);
    let contract = setup_contract(deps.as_mut());

    // Mint a token
    let token_id = "melt".to_string();
    let name = "Melting power".to_string();
    let description = "Allows the owner to melt anyone looking at him or her".to_string();

    let mint_msg = ExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id.clone(),
        owner: String::from("venus"),
        name,
        description: Some(description),
        image: None,
        extension: Extension {},
    });

    let minter = mock_info("creator", &[]);
    contract
        .execute(deps.as_mut(), mock_env(), minter, mint_msg)
        .unwrap();

    let msg = to_binary("You now have the melting power").unwrap();
    let target = String::from("another_contract");
    let send_msg = ExecuteMsg::SendNft {
        contract: target.clone(),
        token_id: token_id.clone(),
        msg: msg.clone(),
    };

    let random = mock_info("random", &[]);
    let err = contract
        .execute(deps.as_mut(), mock_env(), random, send_msg.clone())
        .unwrap_err();
    assert_eq!(
        err,
        ContractError::Unauthorized {
            description: Some(String::from("sender is neither owner nor operator")),
        }
    );

    // but owner can
    let random = mock_info("venus", &[]);
    let res = contract
        .execute(deps.as_mut(), mock_env(), random, send_msg)
        .unwrap();

    let payload = Cw721ReceiveMsg {
        sender: String::from("venus"),
        token_id: token_id.clone(),
        msg,
    };
    let expected = payload.into_cosmos_msg(target.clone()).unwrap();
    // ensure expected serializes as we think it should
    match &expected {
        CosmosMsg::Wasm(WasmMsg::Execute { contract_addr, .. }) => {
            assert_eq!(contract_addr, &target)
        }
        m => panic!("Unexpected message type: {:?}", m),
    }
    // and make sure this is the request sent by the contract
    assert_eq!(
        res,
        Response::new()
            .add_message(expected)
            .add_attribute("action", "send_nft")
            .add_attribute("sender", "venus")
            .add_attribute("recipient", "another_contract")
            .add_attribute("token_id", token_id)
    );
}

#[test]
fn approving_revoking() {
    let mut deps = mock_dependencies(&[]);
    let contract = setup_contract(deps.as_mut());

    // Mint a token
    let token_id = "grow".to_string();
    let name = "Growing power".to_string();
    let description = "Allows the owner to grow anything".to_string();

    let mint_msg = ExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id.clone(),
        owner: String::from("demeter"),
        name,
        description: Some(description),
        image: None,
        extension: Extension {},
    });

    let minter = mock_info("creator", &[]);
    contract
        .execute(deps.as_mut(), mock_env(), minter, mint_msg)
        .unwrap();

    // Give random transferring power
    let approve_msg = ExecuteMsg::Approve {
        spender: String::from("random"),
        token_id: token_id.clone(),
        expires: None,
    };
    let owner = mock_info("demeter", &[]);
    let res = contract
        .execute(deps.as_mut(), mock_env(), owner, approve_msg)
        .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_attribute("action", "approve")
            .add_attribute("sender", "demeter")
            .add_attribute("spender", "random")
            .add_attribute("token_id", token_id.clone())
    );

    // random can now transfer
    let random = mock_info("random", &[]);
    let transfer_msg = ExecuteMsg::TransferNft {
        recipient: String::from("person"),
        token_id: token_id.clone(),
    };
    contract
        .execute(deps.as_mut(), mock_env(), random, transfer_msg)
        .unwrap();

    // Approvals are removed / cleared
    let query_msg = QueryMsg::OwnerOf {
        token_id: token_id.clone(),
        include_expired: None,
    };
    let res: OwnerOfResponse = from_binary(
        &contract
            .query(deps.as_ref(), mock_env(), query_msg.clone())
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        res,
        OwnerOfResponse {
            owner: String::from("person"),
            approvals: vec![],
        }
    );

    // Approve, revoke, and check for empty, to test revoke
    let approve_msg = ExecuteMsg::Approve {
        spender: String::from("random"),
        token_id: token_id.clone(),
        expires: None,
    };
    let owner = mock_info("person", &[]);
    contract
        .execute(deps.as_mut(), mock_env(), owner.clone(), approve_msg)
        .unwrap();

    let revoke_msg = ExecuteMsg::Revoke {
        spender: String::from("random"),
        token_id,
    };
    contract
        .execute(deps.as_mut(), mock_env(), owner, revoke_msg)
        .unwrap();

    // Approvals are now removed / cleared
    let res: OwnerOfResponse = from_binary(
        &contract
            .query(deps.as_ref(), mock_env(), query_msg)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(
        res,
        OwnerOfResponse {
            owner: String::from("person"),
            approvals: vec![],
        }
    );
}

#[test]
fn approving_all_revoking_all() {
    let mut deps = mock_dependencies(&[]);
    let contract = setup_contract(deps.as_mut());

    // Mint a couple tokens (from the same owner)
    let token_id1 = "grow1".to_string();
    let name1 = "Growing power".to_string();
    let description1 = "Allows the owner the power to grow anything".to_string();
    let token_id2 = "grow2".to_string();
    let name2 = "More growing power".to_string();
    let description2 = "Allows the owner the power to grow anything even faster".to_string();

    let mint_msg1 = ExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id1.clone(),
        owner: String::from("demeter"),
        name: name1,
        description: Some(description1),
        image: None,
        extension: Extension {},
    });

    let minter = mock_info("creator", &[]);
    contract
        .execute(deps.as_mut(), mock_env(), minter.clone(), mint_msg1)
        .unwrap();

    let mint_msg2 = ExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id2.clone(),
        owner: String::from("demeter"),
        name: name2,
        description: Some(description2),
        image: None,
        extension: Extension {},
    });

    contract
        .execute(deps.as_mut(), mock_env(), minter, mint_msg2)
        .unwrap();

    // paginate the token_ids
    let tokens = contract.all_tokens(deps.as_ref(), None, Some(1)).unwrap();
    assert_eq!(1, tokens.tokens.len());
    assert_eq!(vec![token_id1.clone()], tokens.tokens);
    let tokens = contract
        .all_tokens(deps.as_ref(), Some(token_id1.clone()), Some(3))
        .unwrap();
    assert_eq!(1, tokens.tokens.len());
    assert_eq!(vec![token_id2.clone()], tokens.tokens);

    // demeter gives random full (operator) power over her tokens
    let approve_all_msg = ExecuteMsg::ApproveAll {
        operator: String::from("random"),
        expires: None,
    };
    let owner = mock_info("demeter", &[]);
    let res = contract
        .execute(deps.as_mut(), mock_env(), owner, approve_all_msg)
        .unwrap();
    assert_eq!(
        res,
        Response::new()
            .add_attribute("action", "approve_all")
            .add_attribute("sender", "demeter")
            .add_attribute("operator", "random")
    );

    // random can now transfer
    let random = mock_info("random", &[]);
    let transfer_msg = ExecuteMsg::TransferNft {
        recipient: String::from("person"),
        token_id: token_id1,
    };
    contract
        .execute(deps.as_mut(), mock_env(), random.clone(), transfer_msg)
        .unwrap();

    // random can now send
    let inner_msg = WasmMsg::Execute {
        contract_addr: "another_contract".into(),
        msg: to_binary("You now also have the growing power").unwrap(),
        funds: vec![],
    };
    let msg: CosmosMsg = CosmosMsg::Wasm(inner_msg);

    let send_msg = ExecuteMsg::SendNft {
        contract: String::from("another_contract"),
        token_id: token_id2,
        msg: to_binary(&msg).unwrap(),
    };
    contract
        .execute(deps.as_mut(), mock_env(), random, send_msg)
        .unwrap();

    // Approve_all, revoke_all, and check for empty, to test revoke_all
    let approve_all_msg = ExecuteMsg::ApproveAll {
        operator: String::from("operator"),
        expires: None,
    };
    // person is now the owner of the tokens
    let owner = mock_info("person", &[]);
    contract
        .execute(deps.as_mut(), mock_env(), owner, approve_all_msg)
        .unwrap();

    let res = contract
        .all_approvals(
            deps.as_ref(),
            mock_env(),
            String::from("person"),
            true,
            None,
            None,
        )
        .unwrap();

    assert_eq!(
        res,
        ApprovedForAllResponse {
            operators: vec![Approval {
                spender: String::from("operator"),
                expires: Expiration::Never {}
            }]
        }
    );

    // second approval
    let buddy_expires = Expiration::AtHeight(1234567);
    let approve_all_msg = ExecuteMsg::ApproveAll {
        operator: String::from("buddy"),
        expires: Some(buddy_expires),
    };
    let owner = mock_info("person", &[]);
    contract
        .execute(deps.as_mut(), mock_env(), owner.clone(), approve_all_msg)
        .unwrap();

    // and paginate queries
    let res = contract
        .all_approvals(
            deps.as_ref(),
            mock_env(),
            String::from("person"),
            true,
            None,
            Some(1),
        )
        .unwrap();
    assert_eq!(
        res,
        ApprovedForAllResponse {
            operators: vec![Approval {
                spender: String::from("buddy"),
                expires: buddy_expires,
            }]
        }
    );
    let res = contract
        .all_approvals(
            deps.as_ref(),
            mock_env(),
            String::from("person"),
            true,
            Some(String::from("buddy")),
            Some(2),
        )
        .unwrap();
    assert_eq!(
        res,
        ApprovedForAllResponse {
            operators: vec![Approval {
                spender: String::from("operator"),
                expires: Expiration::Never {}
            }]
        }
    );

    let revoke_all_msg = ExecuteMsg::RevokeAll {
        operator: String::from("operator"),
    };
    contract
        .execute(deps.as_mut(), mock_env(), owner, revoke_all_msg)
        .unwrap();

    // Approvals are removed / cleared without affecting others
    let res = contract
        .all_approvals(
            deps.as_ref(),
            mock_env(),
            String::from("person"),
            false,
            None,
            None,
        )
        .unwrap();
    assert_eq!(
        res,
        ApprovedForAllResponse {
            operators: vec![Approval {
                spender: String::from("buddy"),
                expires: buddy_expires,
            }]
        }
    );

    // ensure the filter works (nothing should be here
    let mut late_env = mock_env();
    late_env.block.height = 1234568; //expired
    let res = contract
        .all_approvals(
            deps.as_ref(),
            late_env,
            String::from("person"),
            false,
            None,
            None,
        )
        .unwrap();
    assert_eq!(0, res.operators.len());
}

#[test]
fn query_tokens_by_owner() {
    let mut deps = mock_dependencies(&[]);
    let contract = setup_contract(deps.as_mut());
    let minter = mock_info("creator", &[]);

    // Mint a couple tokens (from the same owner)
    let token_id1 = "grow1".to_string();
    let demeter = String::from("Demeter");
    let token_id2 = "grow2".to_string();
    let ceres = String::from("Ceres");
    let token_id3 = "sing".to_string();

    let mint_msg = ExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id1.clone(),
        owner: demeter.clone(),
        name: "Growing power".to_string(),
        description: Some("Allows the owner the power to grow anything".to_string()),
        image: None,
        extension: Extension {},
    });
    contract
        .execute(deps.as_mut(), mock_env(), minter.clone(), mint_msg)
        .unwrap();

    let mint_msg = ExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id2.clone(),
        owner: ceres.clone(),
        name: "More growing power".to_string(),
        description: Some("Allows the owner the power to grow anything even faster".to_string()),
        image: None,
        extension: Extension {},
    });
    contract
        .execute(deps.as_mut(), mock_env(), minter.clone(), mint_msg)
        .unwrap();

    let mint_msg = ExecuteMsg::Mint(MintMsg::<Extension> {
        token_id: token_id3.clone(),
        owner: demeter.clone(),
        name: "Sing a lullaby".to_string(),
        description: Some("Calm even the most excited children".to_string()),
        image: None,
        extension: Extension {},
    });
    contract
        .execute(deps.as_mut(), mock_env(), minter, mint_msg)
        .unwrap();

    // get all tokens in order:
    let expected = vec![token_id1.clone(), token_id2.clone(), token_id3.clone()];
    let tokens = contract.all_tokens(deps.as_ref(), None, None).unwrap();
    assert_eq!(&expected, &tokens.tokens);
    // paginate
    let tokens = contract.all_tokens(deps.as_ref(), None, Some(2)).unwrap();
    assert_eq!(&expected[..2], &tokens.tokens[..]);
    let tokens = contract
        .all_tokens(deps.as_ref(), Some(expected[1].clone()), None)
        .unwrap();
    assert_eq!(&expected[2..], &tokens.tokens[..]);

    // get by owner
    let by_ceres = vec![token_id2];
    let by_demeter = vec![token_id1, token_id3];
    // all tokens by owner
    let tokens = contract
        .tokens(deps.as_ref(), demeter.clone(), None, None)
        .unwrap();
    assert_eq!(&by_demeter, &tokens.tokens);
    let tokens = contract.tokens(deps.as_ref(), ceres, None, None).unwrap();
    assert_eq!(&by_ceres, &tokens.tokens);

    // paginate for demeter
    let tokens = contract
        .tokens(deps.as_ref(), demeter.clone(), None, Some(1))
        .unwrap();
    assert_eq!(&by_demeter[..1], &tokens.tokens[..]);
    let tokens = contract
        .tokens(deps.as_ref(), demeter, Some(by_demeter[0].clone()), Some(3))
        .unwrap();
    assert_eq!(&by_demeter[1..], &tokens.tokens[..]);
}

#[test]
fn test_is_available() {
    let msg = InstantiateMsg {
        name: CONTRACT_NAME.to_string(),
        symbol: SYMBOL.to_string(),
        base_name: BASE_NAME.to_string(),
        base_node: UST_BASE_NODE.to_string(),
        registry_address: String::from("hellooo"),
        grace_period: None,
    };

    let mut deps = mock_dependencies(&[]);
    let info = mock_info("creator", &coins(0, "uusd"));
    let _res = entry::instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let id = String::from("alice");

    let res = entry::query(deps.as_ref(), mock_env(), QueryMsg::IsAvailable { id }).unwrap();

    let value: IsAvailableResponse = from_binary(&res).unwrap();
    assert_eq!(value.available, true);
}

#[test]
fn test_register() {
    let registry_address = String::from("registry_address");

    let msg = InstantiateMsg {
        name: CONTRACT_NAME.to_string(),
        symbol: SYMBOL.to_string(),
        base_node: UST_BASE_NODE.to_string(),
        base_name: BASE_NAME.to_string(),
        registry_address: registry_address.clone(),
        grace_period: None,
    };

    let mut deps = mock_dependencies(&[]);
    let info = mock_info("creator", &coins(0, "uusd"));
    let _res = entry::instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let id = String::from("9c0257114eb9399a2985f8e75dad7600c5d89fe3824ffa99ec1c3eb8bf3b0501");
    let controller = String::from("controller_address");

    let info = mock_info("creator", &coins(0, "uusd"));
    let msg = ExecuteMsg::AddController {
        address: controller.clone(),
    };
    entry::execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let info = mock_info("not_controller_address", &coins(0, "uusd"));
    let msg = ExecuteMsg::Register {
        id: id.clone(),
        owner: controller.clone(),
        duration: 100,
        name: "alice".to_string(),
    };
    assert_eq!(
        entry::execute(deps.as_mut(), mock_env(), info, msg).is_err(),
        true
    );

    let info = mock_info("controller_address", &coins(0, "uusd"));
    let msg = ExecuteMsg::Register {
        id: id.clone(),
        owner: controller.clone(),
        duration: 100,
        name: "alice".to_string(),
    };
    let res = entry::execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let owner_of_query = entry::query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::OwnerOf {
            token_id: id.clone(),
            include_expired: Some(false),
        },
    )
    .unwrap();
    let owner_of_response: OwnerOfResponse = from_binary(&owner_of_query).unwrap();
    assert_eq!(owner_of_response.owner, "controller_address");

    let nft_info_query = entry::query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::NftInfo {
            token_id: id.clone(),
        },
    )
    .unwrap();
    let nft_info_response: NftInfoResponse<Extension> = from_binary(&nft_info_query).unwrap();
    assert_eq!(nft_info_response.name, "alice.ust");

    assert_eq!(res.messages.len(), 1); // set subnode owner

    let set_subnode_owner_registry_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: registry_address.clone(),
        msg: to_binary(&RegistryExecuteMsg::SetSubnodeOwner {
            node: hex::decode(UST_BASE_NODE).unwrap(),
            label: hex::decode(id.clone()).unwrap(),
            owner: controller.clone(),
        })
        .unwrap(),
        funds: vec![],
    });
    assert_eq!(res.messages[0].msg, set_subnode_owner_registry_msg);
}

#[test]
fn test_reclaim() {
    // Setup
    let registry_address = String::from("registry_address");
    let msg = InstantiateMsg {
        name: CONTRACT_NAME.to_string(),
        symbol: SYMBOL.to_string(),
        base_node: UST_BASE_NODE.to_string(),
        base_name: BASE_NAME.to_string(),
        registry_address: registry_address.clone(),
        grace_period: None,
    };
    let mut deps = mock_dependencies(&[]);
    let info = mock_info("creator", &coins(0, "uusd"));
    let _res = entry::instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let id = String::from("9c0257114eb9399a2985f8e75dad7600c5d89fe3824ffa99ec1c3eb8bf3b0501");
    let controller = String::from("controller_address");

    let info = mock_info("creator", &coins(0, "uusd"));
    let msg = ExecuteMsg::AddController {
        address: controller.clone(),
    };
    entry::execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let info = mock_info("controller_address", &coins(0, "uusd"));
    let msg = ExecuteMsg::Register {
        id: id.clone(),
        owner: controller.clone(),
        duration: 100,
        name: "alice".to_string(),
    };
    entry::execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Reclaim from controller to alice
    let info = mock_info("controller_address", &coins(0, "uusd"));
    let msg = ExecuteMsg::Reclaim {
        id: id.clone(),
        owner: "alice".to_string(),
    };
    entry::execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Transfer NFT from controller to alice
    let info = mock_info("controller_address", &coins(0, "uusd"));
    let msg = ExecuteMsg::TransferNft {
        token_id: id.clone(),
        recipient: "alice".to_string(),
    };
    entry::execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Check NFT owner should be alice
    let owner_of_query = entry::query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::OwnerOf {
            token_id: id.clone(),
            include_expired: Some(false),
        },
    )
    .unwrap();
    let owner_of_response: OwnerOfResponse = from_binary(&owner_of_query).unwrap();
    assert_eq!(owner_of_response.owner, "alice");

    // Reclaim alice.ust from bob should error
    let info = mock_info("bob", &coins(0, "uusd"));
    let msg = ExecuteMsg::Reclaim {
        id: id.clone(),
        owner: "bob".to_string(),
    };
    assert_eq!(
        entry::execute(deps.as_mut(), mock_env(), info, msg).is_err(),
        true
    );

    // Reclaim alice.ust from controller should error
    let info = mock_info("controller", &coins(0, "uusd"));
    let msg = ExecuteMsg::Reclaim {
        id: id.clone(),
        owner: "controller".to_string(),
    };
    assert_eq!(
        entry::execute(deps.as_mut(), mock_env(), info, msg).is_err(),
        true
    );

    // Transfer NFT from alice to bob
    let info = mock_info("alice", &coins(0, "uusd"));
    let msg = ExecuteMsg::TransferNft {
        token_id: id.clone(),
        recipient: "bob".to_string(),
    };
    entry::execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Check NFT owner should be bob
    let owner_of_query = entry::query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::OwnerOf {
            token_id: id.clone(),
            include_expired: Some(false),
        },
    )
    .unwrap();
    let owner_of_response: OwnerOfResponse = from_binary(&owner_of_query).unwrap();
    assert_eq!(owner_of_response.owner, "bob");

    // Reclaim alice.ust from alice should error
    let info = mock_info("alice", &coins(0, "uusd"));
    let msg = ExecuteMsg::Reclaim {
        id: id.clone(),
        owner: "controller".to_string(),
    };
    assert_eq!(
        entry::execute(deps.as_mut(), mock_env(), info, msg).is_err(),
        true
    );

    // Reclaim alice.ust from bob
    let info = mock_info("bob", &coins(0, "uusd"));
    let msg = ExecuteMsg::Reclaim {
        id: id.clone(),
        owner: "controller".to_string(),
    };
    entry::execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Reclaim alice.ust from bob
    let info = mock_info("bob", &coins(0, "uusd"));
    let msg = ExecuteMsg::Reclaim {
        id: id.clone(),
        owner: "bob".to_string(),
    };
    entry::execute(deps.as_mut(), mock_env(), info, msg).unwrap();
}

#[test]
fn test_set_config() {
    // Setup
    let registry_address = String::from("registry_address");
    let msg = InstantiateMsg {
        name: CONTRACT_NAME.to_string(),
        symbol: SYMBOL.to_string(),
        base_node: UST_BASE_NODE.to_string(),
        base_name: BASE_NAME.to_string(),
        registry_address: registry_address.clone(),
        grace_period: None,
    };
    let mut deps = mock_dependencies(&[]);
    let info = mock_info("creator", &coins(0, "uusd"));
    let _res = entry::instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let info = mock_info("not_creator", &coins(0, "uusd"));
    let msg = ExecuteMsg::SetConfig {
        grace_period: 3592000,
        registry_address: String::from("new_registry_address"),
        owner: String::from("new_owner"),
    };
    let err = entry::execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
    assert_eq!(
        err,
        ContractError::NotOwner {
            sender: String::from("not_creator"),
            owner: String::from("creator")
        }
    );

    // Set new grace period
    let info = mock_info("creator", &coins(0, "uusd"));
    let msg = ExecuteMsg::SetConfig {
        grace_period: 3592000,
        registry_address: String::from("new_registry_address"),
        owner: String::from("new_owner"),
    };
    entry::execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // Should get new grace period
    let get_config_query = entry::query(deps.as_ref(), mock_env(), QueryMsg::GetConfig {}).unwrap();
    let get_config: ConfigResponse = from_binary(&get_config_query).unwrap();
    assert_eq!(get_config.grace_period, 3592000);
    assert_eq!(
        get_config.registry_address,
        Addr::unchecked(String::from("new_registry_address"))
    );
    assert_eq!(get_config.owner, Addr::unchecked(String::from("new_owner")));
}
