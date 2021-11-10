mod tests {
    use crate::contract::{execute, instantiate, query};
    use crate::error::ContractError;
    use crate::handler::consume_commitment;
    use crate::mock_querier::mock_dependencies;
    use crate::msg::{
        ExecuteMsg, GetCommitmentResponse, InstantiateMsg, MaxCommitmentAgeResponse,
        MinCommitmentAgeResponse, MinRegistrationDurationResponse, NodehashResponse, OwnerResponse,
        PriceResponse, QueryMsg, RegistrarResponse, RentPriceResponse, TokenIdResponse,
    };
    use crate::state::COMMITMENTS;
    use cosmwasm_std::testing::{mock_env, mock_info};
    use cosmwasm_std::{
        coins, from_binary, to_binary, Addr, BankMsg, Coin, CosmosMsg, Timestamp, Uint128, WasmMsg,
    };
    use tns::registrar::{ExecuteMsg as RegistrarExecuteMsg, Extension};
    use tns::registry::ExecuteMsg as RegistryExecuteMsg;
    use tns::resolver::ExecuteMsg as ResolverExecuteMsg;

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);
        let msg = InstantiateMsg {
            registrar_address: String::from("registrar_address"),
            min_commitment_age: 0,
            min_registration_duration: 0,
            max_commitment_age: 0,
            tier1_price: 640_000_000u64,
            tier2_price: 160_000_000u64,
            tier3_price: 5_000_000u64,
            enable_registration: true,
        };
        let info = mock_info("creator", &coins(0, "uusd"));
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    #[test]
    fn test_get_token_id() {
        let mut deps = mock_dependencies(&[]);
        let msg = InstantiateMsg {
            registrar_address: String::from("registrar_address"),
            min_commitment_age: 0,
            min_registration_duration: 0,
            max_commitment_age: 0,
            tier1_price: 640_000_000u64,
            tier2_price: 160_000_000u64,
            tier3_price: 5_000_000u64,
            enable_registration: true,
        };
        let info = mock_info("creator", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let name = String::from("alice");

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetTokenId { name }).unwrap();
        let token_id_response: TokenIdResponse = from_binary(&res).unwrap();
        assert_eq!(
            token_id_response.token_id,
            "9c0257114eb9399a2985f8e75dad7600c5d89fe3824ffa99ec1c3eb8bf3b0501"
        );
    }

    #[test]
    fn test_get_nodehash_from_name() {
        let mut deps = mock_dependencies(&[]);
        let msg = InstantiateMsg {
            registrar_address: String::from("registrar_address"),
            min_commitment_age: 0,
            min_registration_duration: 0,
            max_commitment_age: 0,
            tier1_price: 640_000_000u64,
            tier2_price: 160_000_000u64,
            tier3_price: 5_000_000u64,
            enable_registration: true,
        };
        let info = mock_info("creator", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let name = String::from("alice");
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetNodehash { name }).unwrap();
        let nodehash_response: NodehashResponse = from_binary(&res).unwrap();
        assert_eq!(
            nodehash_response.node,
            [
                78, 137, 50, 222, 163, 237, 87, 141, 30, 30, 144, 123, 133, 152, 167, 161, 204, 44,
                197, 227, 125, 124, 105, 133, 160, 177, 82, 121, 97, 207, 166, 156
            ]
        )
    }

    // Commit
    #[test] //Should be able to commit
    fn test_commit() {
        let mut deps = mock_dependencies(&[]);
        let msg = InstantiateMsg {
            registrar_address: String::from("registrar_address"),
            min_commitment_age: 10,
            min_registration_duration: 10,
            max_commitment_age: 100,
            tier1_price: 640_000_000u64,
            tier2_price: 160_000_000u64,
            tier3_price: 5_000_000u64,
            enable_registration: true,
        };
        let info = mock_info("creator", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let commitment =
            String::from("9232a542ecd323875f2ebac7db9f86ab606badb823af8628b7615ad78227e349");

        let msg = ExecuteMsg::Commit {
            commitment: commitment.clone(),
        };
        let info = mock_info("creator", &coins(0, "uusd"));
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        assert_eq!(commitment, res.attributes[1].value)
    }

    #[test] //Should not be able to recommit if the commitment is not expired yet
    fn test_too_early_recommit_error() {
        let mut deps = mock_dependencies(&[]);
        let msg = InstantiateMsg {
            registrar_address: String::from("registrar_address"),
            min_commitment_age: 10,
            min_registration_duration: 10,
            max_commitment_age: 100,
            tier1_price: 640_000_000u64,
            tier2_price: 160_000_000u64,
            tier3_price: 5_000_000u64,
            enable_registration: true,
        };
        let info = mock_info("creator", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ExecuteMsg::Commit {
            commitment: String::from(
                "9232a542ecd323875f2ebac7db9f86ab606badb823af8628b7615ad78227e349",
            ),
        };
        let info = mock_info("creator", &coins(0, "uusd"));
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        //fast forward 50 seconds
        let mut env = mock_env();
        env.block.time = Timestamp::from_nanos(1_571_797_469_879_305_533);

        let msg = ExecuteMsg::Commit {
            commitment: String::from(
                "9232a542ecd323875f2ebac7db9f86ab606badb823af8628b7615ad78227e349",
            ),
        };
        let info = mock_info("creator", &coins(0, "uusd"));
        let err = execute(deps.as_mut(), env, info, msg).unwrap_err();
        assert_eq!(
            err,
            ContractError::RecommitTooEarly {
                commit_expired: 1571797519,
                current: 1571797469
            }
        );
    }

    #[test] //Should be ablt to recommit after the commitment expires
    fn test_mature_recommit() {
        let mut deps = mock_dependencies(&[]);
        let msg = InstantiateMsg {
            registrar_address: String::from("registrar_address"),
            min_commitment_age: 10,
            min_registration_duration: 10,
            max_commitment_age: 100,
            tier1_price: 640_000_000u64,
            tier2_price: 160_000_000u64,
            tier3_price: 5_000_000u64,
            enable_registration: true,
        };
        let info = mock_info("creator", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let commitment =
            String::from("9232a542ecd323875f2ebac7db9f86ab606badb823af8628b7615ad78227e349");
        let msg = ExecuteMsg::Commit {
            commitment: commitment.clone(),
        };
        let info = mock_info("creator", &coins(0, "uusd"));
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        //fast forward 150 seconds
        let mut env = mock_env();
        env.block.time = Timestamp::from_nanos(1_571_797_569_879_305_533);

        let msg = ExecuteMsg::Commit {
            commitment: commitment.clone(),
        };
        let info = mock_info("creator", &coins(0, "uusd"));
        let res = execute(deps.as_mut(), env, info, msg).unwrap();

        assert_eq!(commitment, res.attributes[1].value)
    }

    // Consume Commit
    #[test] //Should be able to consume and remove commitment
    fn test_consume_commitment() {
        let mut deps = mock_dependencies(&[]);
        let msg = InstantiateMsg {
            registrar_address: String::from("registrar_address"),
            min_commitment_age: 10,
            min_registration_duration: 10,
            max_commitment_age: 100,
            tier1_price: 640_000_000u64,
            tier2_price: 160_000_000u64,
            tier3_price: 5_000_000u64,
            enable_registration: true,
        };
        let info = mock_info("creator", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let commitment =
            String::from("9232a542ecd323875f2ebac7db9f86ab606badb823af8628b7615ad78227e349");
        let msg = ExecuteMsg::Commit {
            commitment: commitment.clone(),
        };
        let info = mock_info("creator", &coins(0, "uusd"));
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Found the commitment
        let commit_time = COMMITMENTS
            .load(deps.as_mut().storage, commitment.clone())
            .unwrap();
        assert_eq!(commit_time, 1571797419);

        //fast forward 50 seconds
        let mut env = mock_env();
        env.block.time = Timestamp::from_nanos(1_571_797_469_879_305_533);
        consume_commitment(deps.as_mut(), env, commitment.clone()).unwrap();

        // Should not found the commitment
        let res = COMMITMENTS.load(deps.as_mut().storage, commitment.clone());
        assert_eq!(res.is_err(), true);
    }

    #[test] //Should return error commitment age is out of range
    fn test_consume_commitment_time_guard() {
        let mut deps = mock_dependencies(&[]);
        let msg = InstantiateMsg {
            registrar_address: String::from("registrar_address"),
            min_commitment_age: 10,
            min_registration_duration: 10,
            max_commitment_age: 100,
            tier1_price: 640_000_000u64,
            tier2_price: 160_000_000u64,
            tier3_price: 5_000_000u64,
            enable_registration: true,
        };
        let info = mock_info("creator", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let commitment =
            String::from("9232a542ecd323875f2ebac7db9f86ab606badb823af8628b7615ad78227e349");
        let msg = ExecuteMsg::Commit {
            commitment: commitment.clone(),
        };
        let info = mock_info("creator", &coins(0, "uusd"));
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Too early
        let err = consume_commitment(deps.as_mut(), mock_env(), commitment.clone()).unwrap_err();
        assert_eq!(
            err,
            ContractError::CommitmentIsTooEarlyOrExpired {
                commit_expired: 1571797419 + 100,
                commit_matured: 1571797419 + 10,
                current: 1571797419,
            }
        );

        // Too late
        //fast forward 150 seconds
        let mut env = mock_env();
        env.block.time = Timestamp::from_nanos(1_571_797_569_879_305_533);

        let err = consume_commitment(deps.as_mut(), env, commitment.clone()).unwrap_err();
        assert_eq!(
            err,
            ContractError::CommitmentIsTooEarlyOrExpired {
                commit_expired: 1571797419 + 100,
                commit_matured: 1571797419 + 10,
                current: 1571797569,
            }
        );
    }

    #[test] //Should return error when consume nonexist commitment
    fn test_consume_commitment_no_commitment() {
        let mut deps = mock_dependencies(&[]);
        let msg = InstantiateMsg {
            registrar_address: String::from("registrar_address"),
            min_commitment_age: 10,
            min_registration_duration: 10,
            max_commitment_age: 100,
            tier1_price: 640_000_000u64,
            tier2_price: 160_000_000u64,
            tier3_price: 5_000_000u64,
            enable_registration: true,
        };
        let info = mock_info("creator", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let err =
            consume_commitment(deps.as_mut(), mock_env(), String::from("nonexist")).unwrap_err();
        assert_eq!(
            err,
            ContractError::ConsumeNonexistCommitment {
                commitment: String::from("nonexist")
            }
        );
    }

    #[test] // Should return correct messages
    fn test_register() {
        let mut deps = mock_dependencies(&[]);
        let registrar_address = String::from("registrar_address");
        let msg = InstantiateMsg {
            registrar_address: registrar_address.clone(),
            min_commitment_age: 0, // For by-pass commitment guard
            min_registration_duration: 24 * 3600 * 365,
            max_commitment_age: 100,
            tier1_price: 640_000_000u64,
            tier2_price: 160_000_000u64,
            tier3_price: 5_000_000u64,
            enable_registration: true,
        };
        let info = mock_info("creator", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let name = String::from("alice");
        let owner = String::from("alice");
        let secret = String::from("tns_secret");
        let resolver = String::from("registry_address");
        let address = String::from("alice_addr");
        let info = mock_info("alice", &coins(0, "uusd"));
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetCommitment {
                name: name.clone(),
                owner: owner.clone(),
                secret: secret.clone(),
                resolver: Some(resolver.clone()),
                address: Some(address.clone()),
            },
        )
        .unwrap();
        let get_commitment_response: GetCommitmentResponse = from_binary(&res).unwrap();

        let msg = ExecuteMsg::Commit {
            commitment: get_commitment_response.commitment,
        };
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let duration: u64 = 24 * 3600 * 365;
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::RentPrice {
                name: name.clone(),
                duration: duration.clone(),
            },
        )
        .unwrap();
        let rent_price_response: RentPriceResponse = from_binary(&res).unwrap();
        let info = mock_info("alice", &coins(rent_price_response.price.u128(), "uusd"));
        let msg = ExecuteMsg::Register {
            name: name.clone(),
            owner: owner.clone(),
            duration: duration.clone(),
            secret: secret.clone(),
            resolver: Some(resolver.clone()),
            address: Some(address.clone()),
        };
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let register_registrar_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: registrar_address.clone(),
            msg: to_binary(&RegistrarExecuteMsg::<Extension>::Register {
                id: String::from(
                    "9c0257114eb9399a2985f8e75dad7600c5d89fe3824ffa99ec1c3eb8bf3b0501",
                ), // token_id of alice
                owner: mock_env().contract.address.to_string(),
                duration: duration.clone(),
                name: name.clone(),
            })
            .unwrap(),
            funds: vec![],
        });

        let registry_set_resolver_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "registry_address".to_string(),
            msg: to_binary(&RegistryExecuteMsg::SetResolver {
                node: vec![
                    78, 137, 50, 222, 163, 237, 87, 141, 30, 30, 144, 123, 133, 152, 167, 161, 204,
                    44, 197, 227, 125, 124, 105, 133, 160, 177, 82, 121, 97, 207, 166, 156,
                ], // nodehash of alice.ust
                resolver: Some("registry_address".to_string()),
            })
            .unwrap(),
            funds: vec![],
        });

        let set_address_resolver_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "registry_address".to_string(),
            msg: to_binary(&ResolverExecuteMsg::SetTerraAddress {
                node: vec![
                    78, 137, 50, 222, 163, 237, 87, 141, 30, 30, 144, 123, 133, 152, 167, 161, 204,
                    44, 197, 227, 125, 124, 105, 133, 160, 177, 82, 121, 97, 207, 166, 156,
                ], // nodehash of alice.ust
                address: address.clone(),
            })
            .unwrap(),
            funds: vec![],
        });

        let reclaim_registrar_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "registrar_address".to_string(),
            msg: to_binary(&RegistrarExecuteMsg::<Extension>::Reclaim {
                id: String::from(
                    "9c0257114eb9399a2985f8e75dad7600c5d89fe3824ffa99ec1c3eb8bf3b0501",
                ), // token_id of alice,
                owner: owner.clone(),
            })
            .unwrap(),
            funds: vec![],
        });

        let transfer_nft_registrar_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "registrar_address".to_string(),
            msg: to_binary(&RegistrarExecuteMsg::<Extension>::TransferNft {
                recipient: owner.clone(),
                token_id: String::from(
                    "9c0257114eb9399a2985f8e75dad7600c5d89fe3824ffa99ec1c3eb8bf3b0501",
                ), // token_id of alice,
            })
            .unwrap(),
            funds: vec![],
        });

        assert_eq!(res.messages.len(), 5); // Register, Set resolver, Set name, Reclaim, Transfer NFT
        assert_eq!(res.messages[0].msg, register_registrar_msg);
        assert_eq!(res.messages[1].msg, registry_set_resolver_msg);
        assert_eq!(res.messages[2].msg, set_address_resolver_msg);
        assert_eq!(res.messages[3].msg, reclaim_registrar_msg);
        assert_eq!(res.messages[4].msg, transfer_nft_registrar_msg);

        let name = String::from("Alice");
        let owner = String::from("alice");
        let secret = String::from("tns_secret");
        let resolver = String::from("registry_address");
        let address = String::from("alice_addr");
        let info = mock_info("alice", &coins(0, "uusd"));
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetCommitment {
                name: name.clone(),
                owner: owner.clone(),
                secret: secret.clone(),
                resolver: Some(resolver.clone()),
                address: Some(address.clone()),
            },
        )
        .unwrap();
        let get_commitment_response: GetCommitmentResponse = from_binary(&res).unwrap();

        let msg = ExecuteMsg::Commit {
            commitment: get_commitment_response.commitment,
        };
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let duration: u64 = 24 * 3600 * 365;
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::RentPrice {
                name: name.clone(),
                duration: duration.clone(),
            },
        )
        .unwrap();
        let rent_price_response: RentPriceResponse = from_binary(&res).unwrap();
        let info = mock_info("alice", &coins(rent_price_response.price.u128(), "uusd"));
        let msg = ExecuteMsg::Register {
            name: name.clone(),
            owner: owner.clone(),
            duration: duration.clone(),
            secret: secret.clone(),
            resolver: Some(resolver.clone()),
            address: Some(address.clone()),
        };
        assert_eq!(execute(deps.as_mut(), mock_env(), info, msg).is_err(), true);
    }

    #[test] // Should return correct messages
    fn test_owner_register() {
        let mut deps = mock_dependencies(&[]);
        let registrar_address = String::from("registrar_address");
        let msg = InstantiateMsg {
            registrar_address: registrar_address.clone(),
            min_commitment_age: 0, // For by-pass commitment guard
            min_registration_duration: 24 * 3600 * 365,
            max_commitment_age: 100,
            tier1_price: 640_000_000u64,
            tier2_price: 160_000_000u64,
            tier3_price: 5_000_000u64,
            enable_registration: true,
        };
        let info = mock_info("creator", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let name = String::from("alice");
        let owner = String::from("alice");
        let resolver = String::from("registry_address");
        let address = String::from("alice_addr");
        let duration: u64 = 24 * 3600 * 365;
        let msg = ExecuteMsg::OwnerRegister {
            name: name.clone(),
            owner: owner.clone(),
            duration: duration.clone(),
            resolver: Some(resolver.clone()),
            address: Some(address.clone()),
        };
        let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
        let register_registrar_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: registrar_address.clone(),
            msg: to_binary(&RegistrarExecuteMsg::<Extension>::Register {
                id: String::from(
                    "9c0257114eb9399a2985f8e75dad7600c5d89fe3824ffa99ec1c3eb8bf3b0501",
                ), // token_id of alice
                owner: mock_env().contract.address.to_string(),
                duration: duration.clone(),
                name: name.clone(),
            })
            .unwrap(),
            funds: vec![],
        });

        let registry_set_resolver_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "registry_address".to_string(),
            msg: to_binary(&RegistryExecuteMsg::SetResolver {
                node: vec![
                    78, 137, 50, 222, 163, 237, 87, 141, 30, 30, 144, 123, 133, 152, 167, 161, 204,
                    44, 197, 227, 125, 124, 105, 133, 160, 177, 82, 121, 97, 207, 166, 156,
                ], // nodehash of alice.ust
                resolver: Some("registry_address".to_string()),
            })
            .unwrap(),
            funds: vec![],
        });

        let set_address_resolver_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "registry_address".to_string(),
            msg: to_binary(&ResolverExecuteMsg::SetTerraAddress {
                node: vec![
                    78, 137, 50, 222, 163, 237, 87, 141, 30, 30, 144, 123, 133, 152, 167, 161, 204,
                    44, 197, 227, 125, 124, 105, 133, 160, 177, 82, 121, 97, 207, 166, 156,
                ], // nodehash of alice.ust
                address: address.clone(),
            })
            .unwrap(),
            funds: vec![],
        });

        let reclaim_registrar_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "registrar_address".to_string(),
            msg: to_binary(&RegistrarExecuteMsg::<Extension>::Reclaim {
                id: String::from(
                    "9c0257114eb9399a2985f8e75dad7600c5d89fe3824ffa99ec1c3eb8bf3b0501",
                ), // token_id of alice,
                owner: owner.clone(),
            })
            .unwrap(),
            funds: vec![],
        });

        let transfer_nft_registrar_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "registrar_address".to_string(),
            msg: to_binary(&RegistrarExecuteMsg::<Extension>::TransferNft {
                recipient: owner.clone(),
                token_id: String::from(
                    "9c0257114eb9399a2985f8e75dad7600c5d89fe3824ffa99ec1c3eb8bf3b0501",
                ), // token_id of alice,
            })
            .unwrap(),
            funds: vec![],
        });

        assert_eq!(res.messages.len(), 5); // Register, Set resolver, Set name, Reclaim, Transfer NFT
        assert_eq!(res.messages[0].msg, register_registrar_msg);
        assert_eq!(res.messages[1].msg, registry_set_resolver_msg);
        assert_eq!(res.messages[2].msg, set_address_resolver_msg);
        assert_eq!(res.messages[3].msg, reclaim_registrar_msg);
        assert_eq!(res.messages[4].msg, transfer_nft_registrar_msg);

        let name = String::from("Alice");
        let owner = String::from("alice");
        let resolver = String::from("registry_address");
        let address = String::from("alice_addr");
        let duration: u64 = 24 * 3600 * 365;
        let msg = ExecuteMsg::OwnerRegister {
            name: name.clone(),
            owner: owner.clone(),
            duration: duration.clone(),
            resolver: Some(resolver.clone()),
            address: Some(address.clone()),
        };
        assert_eq!(execute(deps.as_mut(), mock_env(), info, msg).is_ok(), true);
    }

    #[test]
    fn test_disable_register() {
        let mut deps = mock_dependencies(&[]);
        let registrar_address = String::from("registrar_address");
        let msg = InstantiateMsg {
            registrar_address: registrar_address.clone(),
            min_commitment_age: 0, // For by-pass commitment guard
            min_registration_duration: 24 * 3600 * 365,
            max_commitment_age: 100,
            tier1_price: 640_000_000u64,
            tier2_price: 160_000_000u64,
            tier3_price: 5_000_000u64,
            enable_registration: false,
        };
        let info = mock_info("creator", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let name = String::from("alice");
        let owner = String::from("alice");
        let secret = String::from("tns_secret");
        let resolver = String::from("registry_address");
        let address = String::from("alice_addr");
        let info = mock_info("alice", &coins(0, "uusd"));
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetCommitment {
                name: name.clone(),
                owner: owner.clone(),
                secret: secret.clone(),
                resolver: Some(resolver.clone()),
                address: Some(address.clone()),
            },
        )
        .unwrap();
        let get_commitment_response: GetCommitmentResponse = from_binary(&res).unwrap();

        let msg = ExecuteMsg::Commit {
            commitment: get_commitment_response.commitment,
        };
        assert_eq!(execute(deps.as_mut(), mock_env(), info, msg).is_err(), true);

        let duration: u64 = 24 * 3600 * 365;
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::RentPrice {
                name: name.clone(),
                duration: duration.clone(),
            },
        )
        .unwrap();
        let rent_price_response: RentPriceResponse = from_binary(&res).unwrap();
        let info = mock_info("alice", &coins(rent_price_response.price.u128(), "uusd"));
        let msg = ExecuteMsg::Register {
            name: name.clone(),
            owner: owner.clone(),
            duration: duration.clone(),
            secret: secret.clone(),
            resolver: Some(resolver.clone()),
            address: Some(address.clone()),
        };
        assert_eq!(execute(deps.as_mut(), mock_env(), info, msg).is_err(), true);

        let msg = ExecuteMsg::SetEnableRegistration {
            enable_registration: true,
        };
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("creator", &coins(0, "uusd")),
            msg,
        )
        .unwrap();

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetCommitment {
                name: name.clone(),
                owner: owner.clone(),
                secret: secret.clone(),
                resolver: Some(resolver.clone()),
                address: Some(address.clone()),
            },
        )
        .unwrap();
        let get_commitment_response: GetCommitmentResponse = from_binary(&res).unwrap();
        let msg = ExecuteMsg::Commit {
            commitment: get_commitment_response.commitment,
        };
        let info = mock_info("alice", &coins(rent_price_response.price.u128(), "uusd"));
        assert_eq!(execute(deps.as_mut(), mock_env(), info, msg).is_ok(), true);

        let info = mock_info("alice", &coins(rent_price_response.price.u128(), "uusd"));
        let msg = ExecuteMsg::Register {
            name: name.clone(),
            owner: owner.clone(),
            duration: duration.clone(),
            secret: secret.clone(),
            resolver: Some(resolver.clone()),
            address: Some(address.clone()),
        };
        assert_eq!(execute(deps.as_mut(), mock_env(), info, msg).is_ok(), true);
    }

    #[test] // Should not be able to register with insufficient fund
    fn test_register_with_insufficient_fund() {
        let mut deps = mock_dependencies(&[]);
        let registrar_address = String::from("registrar_address");
        let msg = InstantiateMsg {
            registrar_address: registrar_address.clone(),
            min_commitment_age: 0, // For by-pass commitment guard
            min_registration_duration: 24 * 3600 * 365,
            max_commitment_age: 100,
            tier1_price: 640_000_000u64,
            tier2_price: 160_000_000u64,
            tier3_price: 5_000_000u64,
            enable_registration: true,
        };
        let info = mock_info("creator", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let name = String::from("alice");
        let owner = String::from("alice");
        let secret = String::from("tns_secret");
        let resolver = String::from("registry_address");
        let address = String::from("alice_addr");
        let info = mock_info("alice", &coins(0, "uusd"));
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetCommitment {
                name: name.clone(),
                owner: owner.clone(),
                secret: secret.clone(),
                resolver: Some(resolver.clone()),
                address: Some(address.clone()),
            },
        )
        .unwrap();
        let get_commitment_response: GetCommitmentResponse = from_binary(&res).unwrap();

        let msg = ExecuteMsg::Commit {
            commitment: get_commitment_response.commitment,
        };
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let duration: u64 = 24 * 3600 * 365;
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::RentPrice {
                name: name.clone(),
                duration: duration.clone(),
            },
        )
        .unwrap();

        let rent_price_response: RentPriceResponse = from_binary(&res).unwrap();

        let info = mock_info(
            "alice",
            &coins(rent_price_response.price.u128() / 2, "uusd"),
        );
        let msg = ExecuteMsg::Register {
            name: name.clone(),
            owner: owner.clone(),
            duration: duration.clone(),
            secret: secret.clone(),
            resolver: Some(resolver.clone()),
            address: Some(address.clone()),
        };

        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(
            err,
            ContractError::InsufficientFund {
                amount: Uint128::from(2_500_000u128),
                required: Uint128::from(5_000_000u128),
            }
        )
    }

    #[test] // Should not be able to register without commitment
    fn test_register_without_commitment() {
        let mut deps = mock_dependencies(&[]);
        let registrar_address = String::from("registrar_address");
        let msg = InstantiateMsg {
            registrar_address: registrar_address.clone(),
            min_commitment_age: 0, // For by-pass commitment guard
            min_registration_duration: 24 * 3600 * 365,
            max_commitment_age: 100,
            tier1_price: 640_000_000u64,
            tier2_price: 160_000_000u64,
            tier3_price: 5_000_000u64,
            enable_registration: true,
        };
        let info = mock_info("creator", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let name = String::from("alice");
        let owner = String::from("alice");
        let secret = String::from("tns_secret");
        let resolver = String::from("public_resolver");
        let address = String::from("alice_addr");

        let duration: u64 = 24 * 3600 * 365;
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::RentPrice {
                name: name.clone(),
                duration: duration.clone(),
            },
        )
        .unwrap();
        let rent_price_response: RentPriceResponse = from_binary(&res).unwrap();

        let info = mock_info("alice", &coins(rent_price_response.price.u128(), "uusd"));
        let msg = ExecuteMsg::Register {
            name: name.clone(),
            owner: owner.clone(),
            duration: duration.clone(),
            secret: secret.clone(),
            resolver: Some(resolver.clone()),
            address: Some(address.clone()),
        };

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetCommitment {
                name: name.clone(),
                owner: owner.clone(),
                secret: secret.clone(),
                resolver: Some(resolver.clone()),
                address: Some(address.clone()),
            },
        )
        .unwrap();
        let get_commitment_response: GetCommitmentResponse = from_binary(&res).unwrap();

        // Should error at consume_commitment
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(
            err,
            ContractError::ConsumeNonexistCommitment {
                commitment: get_commitment_response.commitment
            }
        );
    }

    #[test]
    fn test_renew() {
        let mut deps = mock_dependencies(&[]);
        let registrar_address = String::from("registrar_address");
        let msg = InstantiateMsg {
            registrar_address: registrar_address.clone(),
            min_commitment_age: 0, // For by-pass commitment guard
            min_registration_duration: 24 * 3600 * 365,
            max_commitment_age: 100,
            tier1_price: 640_000_000u64,
            tier2_price: 160_000_000u64,
            tier3_price: 5_000_000u64,
            enable_registration: true,
        };
        let info = mock_info("creator", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let name = String::from("alice");
        let owner = String::from("alice");
        let secret = String::from("tns_secret");
        let resolver = String::from("registry_address");
        let address = String::from("alice_addr");
        let info = mock_info("alice", &coins(0, "uusd"));
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetCommitment {
                name: name.clone(),
                owner: owner.clone(),
                secret: secret.clone(),
                resolver: Some(resolver.clone()),
                address: Some(address.clone()),
            },
        )
        .unwrap();
        let get_commitment_response: GetCommitmentResponse = from_binary(&res).unwrap();

        let msg = ExecuteMsg::Commit {
            commitment: get_commitment_response.commitment,
        };
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let duration: u64 = 24 * 3600 * 365;
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::RentPrice {
                name: name.clone(),
                duration: duration.clone(),
            },
        )
        .unwrap();
        let rent_price_response: RentPriceResponse = from_binary(&res).unwrap();

        let info = mock_info("alice", &coins(rent_price_response.price.u128(), "uusd"));
        let msg = ExecuteMsg::Register {
            name: name.clone(),
            owner: owner.clone(),
            duration: duration.clone(),
            secret: secret.clone(),
            resolver: Some(resolver.clone()),
            address: Some(address.clone()),
        };

        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info("alice", &coins(rent_price_response.price.u128(), "uusd"));
        let msg = ExecuteMsg::Renew {
            name: name.clone(),
            duration: duration.clone(),
        };

        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let renew_registrar_message: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "registrar_address".to_string(),
            msg: to_binary(&RegistrarExecuteMsg::<Extension>::Renew {
                id: String::from(
                    "9c0257114eb9399a2985f8e75dad7600c5d89fe3824ffa99ec1c3eb8bf3b0501",
                ), // token_id of alice,,
                duration: duration.clone(),
            })
            .unwrap(),
            funds: vec![],
        });

        assert_eq!(res.messages.len(), 1); // Renew
        assert_eq!(res.messages[0].msg, renew_registrar_message);
    }

    #[test]
    fn test_renew_insufficient_fund() {
        let mut deps = mock_dependencies(&[]);
        let registrar_address = String::from("registrar_address");
        let msg = InstantiateMsg {
            registrar_address: registrar_address.clone(),
            min_commitment_age: 0, // For by-pass commitment guard
            min_registration_duration: 24 * 3600 * 365,
            max_commitment_age: 100,
            tier1_price: 640_000_000u64,
            tier2_price: 160_000_000u64,
            tier3_price: 5_000_000u64,
            enable_registration: true,
        };
        let info = mock_info("creator", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let name = String::from("alice");
        let duration: u64 = 24 * 3600 * 365;
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::RentPrice {
                name: name.clone(),
                duration: duration.clone(),
            },
        )
        .unwrap();
        let rent_price_response: RentPriceResponse = from_binary(&res).unwrap();

        // Sent half of required rent
        let half = rent_price_response.price.u128() / 2;
        let info = mock_info("alice", &coins(half, "uusd"));
        let msg = ExecuteMsg::Renew {
            name: name.clone(),
            duration: duration.clone(),
        };

        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(
            err,
            ContractError::InsufficientFund {
                amount: Uint128::from(half),
                required: Uint128::from(rent_price_response.price.u128())
            }
        );
    }

    #[test]
    fn test_withdraw() {
        let mut deps = mock_dependencies(&[]);
        let registrar_address = String::from("registrar_address");
        let msg = InstantiateMsg {
            registrar_address: registrar_address.clone(),
            min_commitment_age: 0, // For by-pass commitment guard
            min_registration_duration: 24 * 3600 * 365,
            max_commitment_age: 100,
            tier1_price: 640_000_000u64,
            tier2_price: 160_000_000u64,
            tier3_price: 5_000_000u64,
            enable_registration: true,
        };
        let info = mock_info("creator", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info("creator", &coins(0, "uusd"));
        let msg = ExecuteMsg::Withdraw {};

        // Zero balance
        let bank_send_message: CosmosMsg = CosmosMsg::Bank(BankMsg::Send {
            to_address: "creator".to_string(),
            amount: vec![Coin {
                denom: "uusd".to_string(),
                amount: Uint128::from(0 as u32),
            }],
        });

        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        assert_eq!(res.messages.len(), 1); // Renew
        assert_eq!(res.messages[0].msg, bank_send_message);
    }

    #[test] // Should return error if withdraw with non-owner
    fn test_withdraw_not_owner() {
        let mut deps = mock_dependencies(&[]);
        let registrar_address = String::from("registrar_address");
        let msg = InstantiateMsg {
            registrar_address: registrar_address.clone(),
            min_commitment_age: 0, // For by-pass commitment guard
            min_registration_duration: 24 * 3600 * 365,
            max_commitment_age: 100,
            tier1_price: 640_000_000u64,
            tier2_price: 160_000_000u64,
            tier3_price: 5_000_000u64,
            enable_registration: true,
        };
        let info = mock_info("creator", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info("alice", &coins(0, "uusd"));
        let msg = ExecuteMsg::Withdraw {};

        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(
            err,
            ContractError::NotOwner {
                sender: String::from("alice"),
                owner: String::from("creator")
            }
        );
    }

    #[test]
    fn test_set_config() {
        let mut deps = mock_dependencies(&[]);
        let registrar_address = String::from("registrar_address");
        let msg = InstantiateMsg {
            registrar_address: registrar_address.clone(),
            min_commitment_age: 0, // For by-pass commitment guard
            min_registration_duration: 24 * 3600 * 365,
            max_commitment_age: 100,
            tier1_price: 640_000_000u64,
            tier2_price: 160_000_000u64,
            tier3_price: 5_000_000u64,
            enable_registration: true,
        };
        let info = mock_info("creator", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::SetConfig {
            max_commitment_age: 120,
            min_commitment_age: 20,
            min_registration_duration: 24 * 3600 * 365 * 2,
            tier1_price: 6_000_000u64,
            tier2_price: 5_000_000u64,
            tier3_price: 4_000_000u64,
            enable_registration: true,
            registrar_address: String::from("new_registrar_address"),
            owner: String::from("new_owner"),
        };
        execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = QueryMsg::Owner {};
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let res: OwnerResponse = from_binary(&res).unwrap();
        assert_eq!(
            res,
            OwnerResponse {
                owner: Addr::unchecked(String::from("new_owner"))
            }
        );

        let msg = QueryMsg::Registrar {};
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let res: RegistrarResponse = from_binary(&res).unwrap();
        assert_eq!(
            res,
            RegistrarResponse {
                registrar_address: Addr::unchecked(String::from("new_registrar_address")),
            }
        );

        let msg = QueryMsg::MaxCommitmentAge {};
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let res: MaxCommitmentAgeResponse = from_binary(&res).unwrap();
        assert_eq!(res, MaxCommitmentAgeResponse { age: 120 });

        let msg = QueryMsg::MinCommitmentAge {};
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let res: MinCommitmentAgeResponse = from_binary(&res).unwrap();
        assert_eq!(res, MinCommitmentAgeResponse { age: 20 });

        let msg = QueryMsg::MinRegistrationDuration {};
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let res: MinRegistrationDurationResponse = from_binary(&res).unwrap();
        assert_eq!(
            res,
            MinRegistrationDurationResponse {
                duration: 24 * 3600 * 365 * 2,
            }
        );

        let msg = QueryMsg::GetPrice {};
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let res: PriceResponse = from_binary(&res).unwrap();
        assert_eq!(
            res,
            PriceResponse {
                tier1_price: 6_000_000u64,
                tier2_price: 5_000_000u64,
                tier3_price: 4_000_000u64,
            }
        );
    }

    #[test] // Should return error if set config with non-owner
    fn test_cannot_set_config_if_not_owner() {
        let mut deps = mock_dependencies(&[]);
        let registrar_address = String::from("registrar_address");
        let msg = InstantiateMsg {
            registrar_address: registrar_address.clone(),
            min_commitment_age: 0, // For by-pass commitment guard
            min_registration_duration: 24 * 3600 * 365,
            max_commitment_age: 100,
            tier1_price: 640_000_000u64,
            tier2_price: 160_000_000u64,
            tier3_price: 5_000_000u64,
            enable_registration: true,
        };
        let info = mock_info("creator", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::SetConfig {
            max_commitment_age: 120,
            min_commitment_age: 20,
            min_registration_duration: 24 * 3600 * 365 * 2,
            tier1_price: 6_000_000u64,
            tier2_price: 5_000_000u64,
            tier3_price: 4_000_000u64,
            enable_registration: true,
            registrar_address: String::from("new_registrar_address"),
            owner: String::from("new_owner"),
        };
        let info = mock_info("alice", &coins(0, "uusd"));
        let err = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap_err();

        assert_eq!(
            err,
            ContractError::NotOwner {
                sender: String::from("alice"),
                owner: String::from("creator")
            }
        );
    }

    #[test]
    fn test_set_config_transfer_owner() {
        let mut deps = mock_dependencies(&[]);
        let registrar_address = String::from("registrar_address");
        let msg = InstantiateMsg {
            registrar_address: registrar_address.clone(),
            min_commitment_age: 0, // For by-pass commitment guard
            min_registration_duration: 24 * 3600 * 365,
            max_commitment_age: 100,
            tier1_price: 640_000_000u64,
            tier2_price: 160_000_000u64,
            tier3_price: 5_000_000u64,
            enable_registration: true,
        };
        let info = mock_info("creator", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::SetConfig {
            max_commitment_age: 120,
            min_commitment_age: 20,
            min_registration_duration: 24 * 3600 * 365 * 2,
            tier1_price: 6_000_000u64,
            tier2_price: 5_000_000u64,
            tier3_price: 4_000_000u64,
            enable_registration: true,
            registrar_address: String::from("new_registrar_address"),
            owner: String::from("new_owner"),
        };
        execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = QueryMsg::Owner {};
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let res: OwnerResponse = from_binary(&res).unwrap();
        assert_eq!(
            res,
            OwnerResponse {
                owner: Addr::unchecked(String::from("new_owner"))
            }
        );

        let msg = ExecuteMsg::SetConfig {
            max_commitment_age: 120,
            min_commitment_age: 20,
            min_registration_duration: 24 * 3600 * 365 * 2,
            tier1_price: 6_000_000u64,
            tier2_price: 5_000_000u64,
            tier3_price: 4_000_000u64,
            enable_registration: true,
            registrar_address: String::from("new_registrar_address"),
            owner: String::from("creator"),
        };
        let info = mock_info("new_owner", &coins(0, "uusd"));
        execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = QueryMsg::Owner {};
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let res: OwnerResponse = from_binary(&res).unwrap();
        assert_eq!(
            res,
            OwnerResponse {
                owner: Addr::unchecked(String::from("creator"))
            }
        );
    }
}
