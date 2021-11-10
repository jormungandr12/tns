mod tests {
    use crate::contract::{execute, instantiate, query};
    use crate::error::ContractError;
    use crate::mock_querier::mock_dependencies;
    use cosmwasm_std::testing::{mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, Addr};
    use tns::resolver::{
        AddressResponse, ConfigResponse, ContentHashResponse, ExecuteMsg, InstantiateMsg, QueryMsg,
        TextDataResponse,
    };
    use tns::utils::namehash;

    #[test]
    fn test_non_owner_cannot_set_address() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {
            interface_id: 1,
            registry_address: String::from("registry_address"),
        };
        let info = mock_info("not_owner", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ExecuteMsg::SetAddress {
            node: namehash("test.ust"),
            coin_type: 0,
            address: String::from("new_address"),
        };
        let err = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("anyone", &coins(0, "token")),
            msg,
        )
        .unwrap_err();
        assert_eq!(
            err,
            ContractError::NotNodeOwner {
                sender: String::from("anyone"),
                node: format!("{:?}", namehash("test.ust"))
            }
        );
    }

    #[test]
    fn test_set_address() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {
            interface_id: 1,
            registry_address: String::from("registry_address"),
        };
        let info = mock_info("owner", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Set Text Data
        let msg = ExecuteMsg::SetAddress {
            node: namehash("test.ust"),
            address: String::from("new_address"),
            coin_type: 1,
        };
        execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner_address", &coins(0, "token")),
            msg,
        )
        .unwrap();

        let query_msg = QueryMsg::GetAddress {
            node: namehash("test.ust"),
            coin_type: 1,
        };

        let res = query(deps.as_ref(), mock_env(), query_msg.clone()).unwrap();

        let res: AddressResponse = from_binary(&res).unwrap();

        assert_eq!(
            AddressResponse {
                address: String::from("new_address")
            },
            res
        );
    }

    #[test]
    fn test_set_text_data() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {
            interface_id: 1,
            registry_address: String::from("registry_address"),
        };
        let info = mock_info("owner", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Set Text Data
        let msg = ExecuteMsg::SetTextData {
            node: namehash("test.ust"),
            key: String::from("test"),
            value: String::from("1"),
        };
        let _res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner_address", &coins(0, "token")),
            msg,
        )
        .unwrap();

        let query_msg = QueryMsg::GetTextData {
            node: namehash("test.ust"),
            key: String::from("test"),
        };

        let res = query(deps.as_ref(), mock_env(), query_msg.clone()).unwrap();

        let res: TextDataResponse = from_binary(&res).unwrap();
        assert_eq!(
            TextDataResponse {
                data: String::from("1")
            },
            res
        );
    }

    #[test]
    fn test_set_content_hash() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {
            interface_id: 1,
            registry_address: String::from("registry_address"),
        };
        let info = mock_info("owner", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Set Text Data
        let msg = ExecuteMsg::SetContentHash {
            node: (namehash("test.ust")),
            hash: Vec::from("test"),
        };

        let _res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner_address", &coins(0, "token")),
            msg,
        )
        .unwrap();

        let query_msg = QueryMsg::GetContentHash {
            node: (namehash("test.ust")),
        };

        let res = query(deps.as_ref(), mock_env(), query_msg.clone()).unwrap();

        let res: ContentHashResponse = from_binary(&res).unwrap();
        assert_eq!(
            ContentHashResponse {
                hash: Vec::from("test")
            },
            res
        );
    }

    #[test]
    fn test_set_terra_address() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {
            interface_id: 1,
            registry_address: String::from("registry_address"),
        };
        let info = mock_info("owner", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Set address
        let info = mock_info("owner_address", &coins(0, "uusd"));
        let msg = ExecuteMsg::SetTerraAddress {
            node: namehash("alice.ust"),
            address: String::from("new_address"),
        };
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let query_msg = QueryMsg::GetAddress {
            node: namehash("alice.ust"),
            coin_type: 0x8000014a,
        };

        let res = query(deps.as_ref(), mock_env(), query_msg.clone()).unwrap();

        let res: AddressResponse = from_binary(&res).unwrap();

        assert_eq!(
            AddressResponse {
                address: String::from("new_address")
            },
            res
        );
    }

    #[test]
    fn test_set_config() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {
            interface_id: 1,
            registry_address: String::from("registry_address"),
        };
        let info = mock_info("owner", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ExecuteMsg::SetConfig {
            interface_id: 2,
            registry_address: String::from("new_registry_address"),
            owner: String::from("new_owner"),
        };
        let info = mock_info("owner", &coins(0, "uusd"));
        execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = QueryMsg::GetConfig {};
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let res: ConfigResponse = from_binary(&res).unwrap();
        assert_eq!(
            res,
            ConfigResponse {
                interface_id: 2,
                registry_address: Addr::unchecked(String::from("new_registry_address")),
                owner: Addr::unchecked(String::from("new_owner"))
            }
        );
    }

    #[test] // Should return error if set config with non-owner
    fn test_cannot_set_config_if_not_owner() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {
            interface_id: 1,
            registry_address: String::from("registry_address"),
        };
        let info = mock_info("owner", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ExecuteMsg::SetConfig {
            interface_id: 2,
            registry_address: String::from("new_registry_address"),
            owner: String::from("new_owner"),
        };
        let info = mock_info("not_owner", &coins(0, "uusd"));
        let err = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap_err();

        assert_eq!(
            err,
            ContractError::NotOwner {
                sender: String::from("not_owner"),
                owner: String::from("owner")
            }
        );
    }

    #[test]
    fn test_set_config_transfer_owner() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {
            interface_id: 1,
            registry_address: String::from("registry_address"),
        };
        let info = mock_info("owner", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ExecuteMsg::SetConfig {
            interface_id: 2,
            registry_address: String::from("new_registry_address"),
            owner: String::from("new_owner"),
        };
        let info = mock_info("owner", &coins(0, "uusd"));
        execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = QueryMsg::GetConfig {};
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let res: ConfigResponse = from_binary(&res).unwrap();
        assert_eq!(
            res,
            ConfigResponse {
                interface_id: 2,
                registry_address: Addr::unchecked(String::from("new_registry_address")),
                owner: Addr::unchecked(String::from("new_owner"))
            }
        );

        let msg = ExecuteMsg::SetConfig {
            interface_id: 3,
            registry_address: String::from("new_registry_address"),
            owner: String::from("owner"),
        };
        let info = mock_info("new_owner", &coins(0, "uusd"));
        execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = QueryMsg::GetConfig {};
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let res: ConfigResponse = from_binary(&res).unwrap();
        assert_eq!(
            res,
            ConfigResponse {
                interface_id: 3,
                registry_address: Addr::unchecked(String::from("new_registry_address")),
                owner: Addr::unchecked(String::from("owner"))
            }
        );
    }
}
