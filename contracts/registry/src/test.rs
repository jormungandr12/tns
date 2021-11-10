mod tests {
    use crate::contract::{execute, instantiate, query};
    use crate::error::ContractError;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary, Addr};
    use tns::registry::{ExecuteMsg, InstantiateMsg, OperatorResponse, QueryMsg, RecordResponse, ConfigResponse};
    use tns::utils::{convert_namehash_to_hex_string, namehash, keccak256, get_label_from_name};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(0, "uusd"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    // Test with https://swolfeyes.github.io/ethereum-namehash-calculator/
    #[test]
    fn test_namehash() {
        assert_eq!(
            convert_namehash_to_hex_string(namehash("eth")),
            "93cdeb708b7545dc668eb9280176169d1c33cfd8ed6f04690a0bcc88a93fc4ae" // Basenode eth
        );
        assert_eq!(
            convert_namehash_to_hex_string(namehash("ust")),
            "749f2b479b45e5da8e4cbecd926ee9a6f78db5424fa6993b6ecababa5d736b12" // Basenode ust
        );
        assert_eq!(
            convert_namehash_to_hex_string(namehash("alice.eth")),
            "787192fc5378cc32aa956ddfdedbf26b24e8d78e40109add0eea2c1a012c3dec" // alice.eth
        );
        assert_eq!(
            convert_namehash_to_hex_string(namehash("alice.ust")),
            "4e8932dea3ed578d1e1e907b8598a7a1cc2cc5e37d7c6985a0b1527961cfa69c" // alice.ust
        );
        assert_eq!(
            convert_namehash_to_hex_string(namehash("alice.bob.ust")),
            "afe05ee8a06e7f85b476ea21f4b4c0cd8bf5417dc1817989866f558b45bfefe9" // alice.bob.ust
        );
    }

    #[test]
    fn test_set_operator() {
        let mut deps = mock_dependencies(&[]);
        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Set up
        let info = mock_info("creator", &coins(0, "uusd"));
        let msg = ExecuteMsg::SetSubnodeOwner {
            node: vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ],
            label: get_label_from_name(&String::from("ust")),
            owner: String::from("owner_address"),
        };
        execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Try to get operator
        let query_msg = QueryMsg::GetIsApprovedForAll {
            owner: String::from("owner_address"),
            operator: String::from("operator_address"),
        };
        let res = query(deps.as_ref(), mock_env(), query_msg.clone()).unwrap();
        let res: OperatorResponse = from_binary(&res).unwrap();
        assert_eq!(OperatorResponse { is_approve: false }, res);

        // Set Operator to true
        let msg = ExecuteMsg::SetApprovalForAll {
            node: namehash("ust"),
            operator: String::from("operator_address"),
            approved: true,
        };
        let _res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner_address", &coins(0, "token")),
            msg,
        )
        .unwrap();

        // Try to get operator
        let query_msg = QueryMsg::GetIsApprovedForAll {
            owner: String::from("owner_address"),
            operator: String::from("operator_address"),
        };
        let res = query(deps.as_ref(), mock_env(), query_msg.clone()).unwrap();
        let res: OperatorResponse = from_binary(&res).unwrap();
        assert_eq!(OperatorResponse { is_approve: true }, res);

        // Set Operator to false with operator
        let msg = ExecuteMsg::SetApprovalForAll {
            node: namehash("ust"),
            operator: String::from("operator_address"),
            approved: false,
        };
        let _res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner_address", &coins(0, "token")),
            msg,
        )
        .unwrap();

        // Try to get operator
        let res = query(deps.as_ref(), mock_env(), query_msg.clone()).unwrap();
        let res: OperatorResponse = from_binary(&res).unwrap();
        assert_eq!(OperatorResponse { is_approve: false }, res);

         // Non operator should not be able to set record
         let info = mock_info("not_operator_address", &coins(0, "uusd"));
         let msg = ExecuteMsg::SetRecord {
             node: namehash("ust"),
             owner: String::from("not_operator_address"),
             resolver: Some(String::from("resolver_address")),
             ttl: 1
         };
         let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
         assert_eq!(err, ContractError::NotNodeOwner {
            sender: String::from("not_operator_address"),
            node: format!("{:?}", namehash("ust"))
        });

        // Set operator to true
        let msg = ExecuteMsg::SetApprovalForAll {
            node: namehash("ust"),
            operator: String::from("operator_address"),
            approved: true,
        };
        let _res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info("owner_address", &coins(0, "token")),
            msg,
        )
        .unwrap();

        // Operator should be able to set record and become new owner
        let info = mock_info("operator_address", &coins(0, "uusd"));
        let msg = ExecuteMsg::SetRecord {
            node: namehash("ust"),
            owner: String::from("operator_address"),
            resolver: Some(String::from("resolver_address")),
            ttl: 1
        };
        assert_eq!(execute(deps.as_mut(), mock_env(), info, msg).is_ok(), true);

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetRecord {
                name: String::from("ust"),
            },
        )
        .unwrap();
        let value: RecordResponse = from_binary(&res).unwrap();
        assert_eq!(
            RecordResponse {
                owner: Addr::unchecked("operator_address"),
                resolver: Addr::unchecked("resolver_address"),
                ttl: 1
            },
            value
        );
    }

    #[test]
    fn test_set_record() {
        let mut deps = mock_dependencies(&[]);
        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Add UST permanent registrar
        let info = mock_info("creator", &coins(0, "uusd"));
        let msg = ExecuteMsg::SetSubnodeOwner {
            node: vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ],
            label: get_label_from_name(&String::from("ust")),
            owner: String::from("registrar_address"),
        };
        assert_eq!(execute(deps.as_mut(), mock_env(), info, msg).is_ok(), true);

        // Check .ust record
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetRecord {
                name: String::from("ust"),
            },
        )
        .unwrap();
        let value: RecordResponse = from_binary(&res).unwrap();
        assert_eq!(
            RecordResponse {
                owner: Addr::unchecked("registrar_address"),
                resolver: mock_env().contract.address,
                ttl: 0
            },
            value
        );

        // Register alice.ust
        let info = mock_info("registrar_address", &coins(0, "uusd"));
        let msg = ExecuteMsg::SetSubnodeOwner {
            node: namehash("ust"), // .ust basenode
            label: get_label_from_name(&String::from("alice")), // alice label, // alice label
            owner: String::from("controller_address"),
        };
        assert_eq!(execute(deps.as_mut(), mock_env(), info, msg).is_ok(), true);

        // Check alice.ust record
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetRecord {
                name: String::from("alice.ust"),
            },
        )
        .unwrap();
        let value: RecordResponse = from_binary(&res).unwrap();
        assert_eq!(
            RecordResponse {
                owner: Addr::unchecked("controller_address"),
                resolver: mock_env().contract.address,
                ttl: 0
            },
            value
        );

        // Cannot set record if sender is not owner
        let info = mock_info("not_controller_address", &coins(0, "uusd"));
        let msg = ExecuteMsg::SetRecord {
            node: namehash("alice.ust"),
            owner: String::from("not_controller_address"),
            resolver: Some(String::from("resolver_address")),
            ttl: 1
        };
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(err, ContractError::NotNodeOwner {
            sender: String::from("not_controller_address"),
            node: format!("{:?}", namehash("alice.ust"))
        });

        // Can set record if sender is owner
        let info = mock_info("controller_address", &coins(0, "uusd"));
        let msg = ExecuteMsg::SetRecord {
            node: namehash("alice.ust"),
            owner: String::from("controller_address"),
            resolver: Some(String::from("resolver_address")),
            ttl: 1
        };
        assert_eq!(execute(deps.as_mut(), mock_env(), info, msg).is_ok(), true);

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetRecord {
                name: String::from("alice.ust"),
            },
        )
        .unwrap();
        let value: RecordResponse = from_binary(&res).unwrap();
        assert_eq!(
            RecordResponse {
                owner: Addr::unchecked("controller_address"),
                resolver: Addr::unchecked("resolver_address"),
                ttl: 1
            },
            value
        );
    }

    #[test]
    fn test_set_subnode_owner() {
        let mut deps = mock_dependencies(&[]);
        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        // Do not pass authorised
        let info = mock_info("registrar_address", &coins(0, "uusd"));
        let msg = ExecuteMsg::SetSubnodeOwner {
            node: namehash("ust"), // .ust basenode
            label: get_label_from_name(&String::from("alice")), // alice label, // alice label
            owner: String::from("registrar_address"),
        };
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(err, ContractError::NotNodeOwner {
            sender: String::from("registrar_address"),
            node: format!("{:?}", namehash("ust"))
        });

        // Add UST permanent registrar
        let info = mock_info("not-creator", &coins(0, "uusd"));
        let msg = ExecuteMsg::SetSubnodeOwner {
            node: vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ],
            label: get_label_from_name(&String::from("ust")),
            owner: String::from("registrar_address"),
        };
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(err, ContractError::NotNodeOwner {
            sender: String::from("not-creator"),
            node: format!("{:?}", vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ])
        });

        let info = mock_info("creator", &coins(0, "uusd"));
        let msg = ExecuteMsg::SetSubnodeOwner {
            node: vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ],
            label: get_label_from_name(&String::from("ust")),
            owner: String::from("registrar_address"),
        };
        assert_eq!(execute(deps.as_mut(), mock_env(), info, msg).is_ok(), true);

        // Check .ust record
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetRecord {
                name: String::from("ust"),
            },
        )
        .unwrap();
        let value: RecordResponse = from_binary(&res).unwrap();
        assert_eq!(
            RecordResponse {
                owner: Addr::unchecked("registrar_address"),
                resolver: mock_env().contract.address,
                ttl: 0
            },
            value
        );

        // Register alice.ust with not registrar address should fail
        let info = mock_info("not-registrar", &coins(0, "uusd"));
        let msg = ExecuteMsg::SetSubnodeOwner {
            node: namehash("ust"), // .ust basenode
            label: get_label_from_name(&String::from("alice")), // alice label, // alice label
            owner: String::from("controller_address"),
        };
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(err, ContractError::NotNodeOwner {
            sender: String::from("not-registrar"),
            node: format!("{:?}", namehash("ust"))
        });

        // Register alice.ust
        let info = mock_info("registrar_address", &coins(0, "uusd"));
        let msg = ExecuteMsg::SetSubnodeOwner {
            node: namehash("ust"), // .ust basenode
            label: get_label_from_name(&String::from("alice")), // alice label, // alice label
            owner: String::from("controller_address"),
        };
        assert_eq!(execute(deps.as_mut(), mock_env(), info, msg).is_ok(), true);

        // Check alice.ust record
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetRecord {
                name: String::from("alice.ust"),
            },
        )
        .unwrap();
        let value: RecordResponse = from_binary(&res).unwrap();
        assert_eq!(
            RecordResponse {
                owner: Addr::unchecked("controller_address"),
                resolver: mock_env().contract.address,
                ttl: 0
            },
            value
        );
    }

    #[test]
    fn test_set_owner() {
        let mut deps = mock_dependencies(&[]);
        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Add UST permanent registrar
        let info = mock_info("creator", &coins(0, "uusd"));
        let msg = ExecuteMsg::SetSubnodeOwner {
            node: vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ],
            label: get_label_from_name(&String::from("ust")),
            owner: String::from("registrar_address"),
        };
        assert_eq!(execute(deps.as_mut(), mock_env(), info, msg).is_ok(), true);

        // Check .ust record
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetRecord {
                name: String::from("ust"),
            },
        )
        .unwrap();
        let value: RecordResponse = from_binary(&res).unwrap();
        assert_eq!(
            RecordResponse {
                owner: Addr::unchecked("registrar_address"),
                resolver: mock_env().contract.address,
                ttl: 0
            },
            value
        );

        // Register alice.ust
        let info = mock_info("registrar_address", &coins(0, "uusd"));
        let msg = ExecuteMsg::SetSubnodeOwner {
            node: namehash("ust"), // .ust basenode
            label: get_label_from_name(&String::from("alice")), // alice label
            owner: String::from("controller_address"),
        };
        assert_eq!(execute(deps.as_mut(), mock_env(), info, msg).is_ok(), true);

        // Check alice.ust record
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetRecord {
                name: String::from("alice.ust"),
            },
        )
        .unwrap();
        let value: RecordResponse = from_binary(&res).unwrap();
        assert_eq!(
            RecordResponse {
                owner: Addr::unchecked("controller_address"),
                resolver: mock_env().contract.address,
                ttl: 0
            },
            value
        );

        // Set resolver
        let node = namehash("ust"); // .ust basenode
        let label = get_label_from_name(&String::from("alice")); // alice label
        let subnode = keccak256(&[node, label].concat());

        //  Should fail if set ttl with non owner address
        let info = mock_info("not_controller_address", &coins(0, "uusd"));
        let msg = ExecuteMsg::SetOwner { node: subnode.clone(), owner: String::from("new_owner") };
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(err, ContractError::NotNodeOwner {
            sender: String::from("not_controller_address"),
            node: format!("{:?}", subnode)
        });

         //  Should success if set ttl with owner address
         let info = mock_info("controller_address", &coins(0, "uusd"));
         let msg = ExecuteMsg::SetOwner { node: subnode.clone(), owner: String::from("new_owner") };
         assert_eq!(execute(deps.as_mut(), mock_env(), info, msg).is_ok(), true);

         // Check alice.ust record
         let res = query(
             deps.as_ref(),
             mock_env(),
             QueryMsg::GetRecord {
                 name: String::from("alice.ust"),
             },
         )
         .unwrap();
         let value: RecordResponse = from_binary(&res).unwrap();
         assert_eq!(
             RecordResponse {
                 owner: Addr::unchecked("new_owner"),
                 resolver: Addr::unchecked("cosmos2contract"),
                 ttl: 0
             },
             value
         );
    }

    #[test]
    fn test_set_ttl() {
        let mut deps = mock_dependencies(&[]);
        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Add UST permanent registrar
        let info = mock_info("creator", &coins(0, "uusd"));
        let msg = ExecuteMsg::SetSubnodeOwner {
            node: vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ],
            label: get_label_from_name(&String::from("ust")),
            owner: String::from("registrar_address"),
        };
        assert_eq!(execute(deps.as_mut(), mock_env(), info, msg).is_ok(), true);

        // Check .ust record
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetRecord {
                name: String::from("ust"),
            },
        )
        .unwrap();
        let value: RecordResponse = from_binary(&res).unwrap();
        assert_eq!(
            RecordResponse {
                owner: Addr::unchecked("registrar_address"),
                resolver: mock_env().contract.address,
                ttl: 0
            },
            value
        );

        // Register alice.ust
        let info = mock_info("registrar_address", &coins(0, "uusd"));
        let msg = ExecuteMsg::SetSubnodeOwner {
            node: namehash("ust"), // .ust basenode
            label: get_label_from_name(&String::from("alice")), // alice label
            owner: String::from("controller_address"),
        };
        assert_eq!(execute(deps.as_mut(), mock_env(), info, msg).is_ok(), true);

        // Check alice.ust record
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetRecord {
                name: String::from("alice.ust"),
            },
        )
        .unwrap();
        let value: RecordResponse = from_binary(&res).unwrap();
        assert_eq!(
            RecordResponse {
                owner: Addr::unchecked("controller_address"),
                resolver: mock_env().contract.address,
                ttl: 0
            },
            value
        );

        // Set resolver
        let node = namehash("ust"); // .ust basenode
        let label = get_label_from_name(&String::from("alice")); // alice label
        let subnode = keccak256(&[node, label].concat());

        //  Should fail if set ttl with non owner address
        let info = mock_info("not_controller_address", &coins(0, "uusd"));
        let msg = ExecuteMsg::SetTTL { node: subnode.clone(), ttl: 3 };
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(err, ContractError::NotNodeOwner {
            sender: String::from("not_controller_address"),
            node: format!("{:?}", subnode)
        });

         //  Should success if set ttl with owner address
         let info = mock_info("controller_address", &coins(0, "uusd"));
         let msg = ExecuteMsg::SetTTL { node: subnode.clone(), ttl: 3 };
         assert_eq!(execute(deps.as_mut(), mock_env(), info, msg).is_ok(), true);

         // Check alice.ust record
         let res = query(
             deps.as_ref(),
             mock_env(),
             QueryMsg::GetRecord {
                 name: String::from("alice.ust"),
             },
         )
         .unwrap();
         let value: RecordResponse = from_binary(&res).unwrap();
         assert_eq!(
             RecordResponse {
                 owner: Addr::unchecked("controller_address"),
                 resolver: Addr::unchecked("cosmos2contract"),
                 ttl: 3
             },
             value
         );
    }

    #[test]
    fn test_set_resolver() {
        let mut deps = mock_dependencies(&[]);
        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Add UST permanent registrar
        let info = mock_info("creator", &coins(0, "uusd"));
        let msg = ExecuteMsg::SetSubnodeOwner {
            node: vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ],
            label: get_label_from_name(&String::from("ust")),
            owner: String::from("registrar_address"),
        };
        assert_eq!(execute(deps.as_mut(), mock_env(), info, msg).is_ok(), true);

        // Check .ust record
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetRecord {
                name: String::from("ust"),
            },
        )
        .unwrap();
        let value: RecordResponse = from_binary(&res).unwrap();
        assert_eq!(
            RecordResponse {
                owner: Addr::unchecked("registrar_address"),
                resolver: mock_env().contract.address,
                ttl: 0
            },
            value
        );

        // Register alice.ust
        let info = mock_info("registrar_address", &coins(0, "uusd"));
        let msg = ExecuteMsg::SetSubnodeOwner {
            node: namehash("ust"), // .ust basenode
            label: get_label_from_name(&String::from("alice")), // alice label
            owner: String::from("controller_address"),
        };
        assert_eq!(execute(deps.as_mut(), mock_env(), info, msg).is_ok(), true);

        // Check alice.ust record
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetRecord {
                name: String::from("alice.ust"),
            },
        )
        .unwrap();
        let value: RecordResponse = from_binary(&res).unwrap();
        assert_eq!(
            RecordResponse {
                owner: Addr::unchecked("controller_address"),
                resolver: mock_env().contract.address,
                ttl: 0
            },
            value
        );

        // Set resolver
        let node = namehash("ust"); // .ust basenode
        let label = get_label_from_name(&String::from("alice")); // alice label
        let subnode = keccak256(&[node, label].concat());

        //  Should fail if set resolver with non owner address
        let info = mock_info("not_controller_address", &coins(0, "uusd"));
        let msg = ExecuteMsg::SetResolver { node: subnode.clone(), resolver: Some(String::from("new_resolver_address")) };
        let err = execute(deps.as_mut(), mock_env(), info, msg).unwrap_err();
        assert_eq!(err, ContractError::NotNodeOwner {
            sender: String::from("not_controller_address"),
            node: format!("{:?}", subnode)
        });

         //  Should success if set resolver with owner address
         let info = mock_info("controller_address", &coins(0, "uusd"));
         let msg = ExecuteMsg::SetResolver { node: subnode.clone(), resolver: Some(String::from("new_resolver_address")) };
         assert_eq!(execute(deps.as_mut(), mock_env(), info, msg).is_ok(), true);

         // Check alice.ust record
         let res = query(
             deps.as_ref(),
             mock_env(),
             QueryMsg::GetRecord {
                 name: String::from("alice.ust"),
             },
         )
         .unwrap();
         let value: RecordResponse = from_binary(&res).unwrap();
         assert_eq!(
             RecordResponse {
                 owner: Addr::unchecked("controller_address"),
                 resolver: Addr::unchecked("new_resolver_address"),
                 ttl: 0
             },
             value
         );
    }

    #[test]
    fn test_get_record_by_node() {
        let mut deps = mock_dependencies(&[]);
        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Add UST permanent registrar
        let info = mock_info("creator", &coins(0, "uusd"));
        let msg = ExecuteMsg::SetSubnodeOwner {
            node: vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ],
            label: get_label_from_name(&String::from("ust")),
            owner: String::from("registrar_address"),
        };
        assert_eq!(execute(deps.as_mut(), mock_env(), info, msg).is_ok(), true);

        // Register alice.ust
        let info = mock_info("registrar_address", &coins(0, "uusd"));
        let msg = ExecuteMsg::SetSubnodeOwner {
            node: namehash("ust"), // .ust basenode
            label: get_label_from_name(&String::from("alice")), // alice label
            owner: String::from("controller_address"),
        };
        assert_eq!(execute(deps.as_mut(), mock_env(), info, msg).is_ok(), true);

        // Check alice.ust record
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetRecord {
                name: String::from("alice.ust"),
            },
        )
        .unwrap();
        let value: RecordResponse = from_binary(&res).unwrap();
        assert_eq!(value.owner, String::from("controller_address"));

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetRecordByNode {
                node: namehash("alice.ust")
            },
        )
        .unwrap();
        let value: RecordResponse = from_binary(&res).unwrap();
        assert_eq!(value.owner, String::from("controller_address"));
    }

    #[test]
    fn test_is_node_owner() {
        let mut deps = mock_dependencies(&[]);
        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // Add UST permanent registrar
        let info = mock_info("creator", &coins(0, "uusd"));
        let msg = ExecuteMsg::SetSubnodeOwner {
            node: vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0,
            ],
            label: get_label_from_name(&String::from("ust")),
            owner: String::from("registrar_address"),
        };
        assert_eq!(execute(deps.as_mut(), mock_env(), info, msg).is_ok(), true);

        // Register alice.ust
        let info = mock_info("registrar_address", &coins(0, "uusd"));
        let msg = ExecuteMsg::SetSubnodeOwner {
            node: namehash("ust"), // .ust basenode
            label: get_label_from_name(&String::from("alice")), // alice label
            owner: String::from("controller_address"),
        };
        assert_eq!(execute(deps.as_mut(), mock_env(), info, msg).is_ok(), true);

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetIsNodeOwner {
                node: namehash("alice.ust"), // alice.ust node
                address: String::from("controller_address"),
            },
        )
        .unwrap();
        let value: bool = from_binary(&res).unwrap();
        assert_eq!(value, true);

        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetIsNodeOwner {
                node: namehash("alice.ust"), // alice.ust node
                address: String::from("not_controller_address"),
            },
        )
        .unwrap();
        let value: bool = from_binary(&res).unwrap();
        assert_eq!(value, false);
    }

    #[test]
    fn test_set_config() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {};
        let info = mock_info("owner", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ExecuteMsg::SetConfig {
            default_resolver: String::from("new_resolver_address"),
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
                default_resolver: Addr::unchecked(String::from("new_resolver_address")),
                owner: Addr::unchecked(String::from("new_owner"))
            }
        );
    }

    #[test] // Should return error if set config with non-owner
    fn test_cannot_set_config_if_not_owner() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {};
        let info = mock_info("owner", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ExecuteMsg::SetConfig {
            default_resolver: String::from("new_resolver_address"),
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

        let msg = InstantiateMsg {};
        let info = mock_info("owner", &coins(0, "uusd"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ExecuteMsg::SetConfig {
            default_resolver: String::from("new_resolver_address"),
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
                default_resolver: Addr::unchecked(String::from("new_resolver_address")),
                owner: Addr::unchecked(String::from("new_owner"))
            }
        );

        let msg = ExecuteMsg::SetConfig {
            default_resolver: String::from("new_resolver_address"),
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
                default_resolver: Addr::unchecked(String::from("new_resolver_address")),
                owner: Addr::unchecked(String::from("owner"))
            }
        );
    }
}
