use near_api::{AccountId, Contract, NearToken};
use near_sandbox::config::{DEFAULT_GENESIS_ACCOUNT, DEFAULT_GENESIS_ACCOUNT_PRIVATE_KEY};
use near_sdk::base64::{engine::general_purpose, Engine as _};
use near_sdk::env::sha256_array;
use near_sdk::json_types::Base58CryptoHash;
use near_sdk::serde_json::{json, Value};
use near_sdk::AccountIdRef;
use std::fs;

#[tokio::test]
async fn test_upgrade() -> testresult::TestResult {
    const SPUTNIKDAO_FACTORY_CONTRACT_ACCOUNT: &AccountIdRef =
        AccountIdRef::new_or_panic("sputnik-dao.near");

    let rpc = near_api::NetworkConfig::mainnet()
        .rpc_endpoints
        .first()
        .unwrap()
        .url
        .clone();

    // Import the mainnet sputnik-dao.near factory contract
    let sputnikdao_factory_contract =
        near_api::Contract(SPUTNIKDAO_FACTORY_CONTRACT_ACCOUNT.to_owned());

    let sandbox = near_sandbox::Sandbox::start_sandbox().await?;
    let sandbox_network =
        near_api::NetworkConfig::from_rpc_url("sandbox", sandbox.rpc_addr.parse()?);
    let signer = near_api::Signer::new(near_api::Signer::from_secret_key(
        DEFAULT_GENESIS_ACCOUNT_PRIVATE_KEY.parse()?,
    ))?;

    let user_account_id: AccountId = format!("some_account.{}", DEFAULT_GENESIS_ACCOUNT).parse()?;
    sandbox
        .create_account(user_account_id.clone())
        .initial_balance(NearToken::from_near(50))
        .send()
        .await?;

    sandbox
        .import_account(rpc.as_str(), sputnikdao_factory_contract.0.clone())
        .initial_balance(NearToken::from_near(100))
        .send()
        .await?;

    // Initialize the sputnik-dao factory contract
    sputnikdao_factory_contract
        .call_function("new", ())?
        .transaction()
        .max_gas()
        .with_signer(sputnikdao_factory_contract.0.clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    // Create a testdao.sputnik-dao.near instance
    let dao_name = "testdao";
    let create_dao_args = json!({
        "config": {
            "name": dao_name,
            "purpose": "creating dao treasury",
            "metadata": "",
        },
        "policy": {
            "roles": [
                {
                    "kind": {
                        "Group": [user_account_id],
                    },
                    "name": "Create Requests",
                    "permissions": [
                        "*:AddProposal"
                    ],
                    "vote_policy": {},
                },
                {
                    "kind": {
                        "Group": [user_account_id],
                    },
                    "name": "Manage Members",
                    "permissions": [
                        "config:*",
                        "policy:*",
                        "add_member_to_role:*",
                        "remove_member_from_role:*",
                    ],
                    "vote_policy": {},
                },
                {
                    "kind": {
                        "Group": [user_account_id],
                    },
                    "name": "Vote",
                    "permissions": ["*:VoteReject", "*:VoteApprove", "*:VoteRemove"],
                    "vote_policy": {},
                },
            ],
            "default_vote_policy": {
                "weight_kind": "RoleWeight",
                "quorum": "0",
                "threshold": [1, 2],
            },
            "proposal_bond": NearToken::from_near(1),
            "proposal_period": "604800000000000",
            "bounty_bond": "100000000000000000000000",
            "bounty_forgiveness_period": "604800000000000",
        },
    });

    sputnikdao_factory_contract
        .call_function(
            "create",
            json!({
                "name": dao_name,
                "args": general_purpose::STANDARD.encode(create_dao_args.to_string())
            }),
        )?
        .transaction()
        .max_gas()
        .deposit(NearToken::from_near(6))
        .with_signer(user_account_id.clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    let dao_account_id: AccountId =
        format!("{}.{}", dao_name, SPUTNIKDAO_FACTORY_CONTRACT_ACCOUNT).parse()?;
    let dao_contract = near_api::Contract(dao_account_id);

    // Verify the DAO configuration
    let config: Value = dao_contract
        .call_function("get_config", ())?
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;
    assert_eq!(create_dao_args["config"], config);

    // Deploy the local build of the sputnik-dao factory contract
    let wasm = fs::read("./res/sputnikdao_factory2.wasm").expect("Unable to read contract wasm");
    Contract::deploy(sputnikdao_factory_contract.0.clone())
        .use_code(wasm.to_vec())
        .without_init_call()
        .with_signer(signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    // Store the local build of sputnikdao2.wasm into sputnik-dao.near
    let sputnikdao2_wasm =
        fs::read("../sputnikdao2/res/sputnikdao2.wasm").expect("Unable to read sputnikdao2.wasm");
    let computed_hash = sha256_array(&sputnikdao2_wasm);
    let stored_contract_hash: Base58CryptoHash = sputnikdao_factory_contract
        .call_function_raw("store", sputnikdao2_wasm.clone())
        .transaction()
        .max_gas()
        .deposit(NearToken::from_near(20))
        .with_signer(sputnikdao_factory_contract.0.clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .json()?;

    // Set the stored contract hash as the default code hash
    sputnikdao_factory_contract
        .call_function(
            "set_default_code_hash",
            json!({"code_hash": stored_contract_hash}),
        )?
        .transaction()
        .with_signer(sputnikdao_factory_contract.0.clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    // Verify the default code hash matches the computed hash
    let hash: Base58CryptoHash = sputnikdao_factory_contract
        .call_function("get_default_code_hash", ())?
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;

    assert_eq!(hash, computed_hash.into(), "Hashes do not match");

    // Create a self-upgrade proposal
    let proposal_id: u64 = dao_contract
        .call_function(
            "add_proposal",
            json!({ "proposal": {
                "description": "proposal to test",
                "kind": {
                    "UpgradeSelf": {
                        "hash": stored_contract_hash
                    }
                }
            }}),
        )?
        .transaction()
        .max_gas()
        .deposit(NearToken::from_near(1))
        .with_signer(user_account_id.clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .json()?;

    assert_eq!(0, proposal_id);

    // Create a transfer proposal to check after the upgrade
    let transfer_proposal_id: u64 = dao_contract
        .call_function(
            "add_proposal",
            json!({ "proposal": {
                "description": "a transfer proposal to check after upgrade",
                "kind": {
                    "Transfer": {
                        "token_id": "",
                        "receiver_id": user_account_id,
                        "amount": "1"
                    },
                },
            }}),
        )?
        .transaction()
        .max_gas()
        .deposit(NearToken::from_near(1))
        .with_signer(user_account_id.clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .json()?;

    // Act on the self-upgrade proposal
    let act_proposal_result = dao_contract
        .call_function(
            "act_proposal",
            json!({
                "id": 0,
                "action": "VoteApprove",
                "proposal": {
                    "UpgradeSelf": {
                        "hash": stored_contract_hash
                    }
                }
            }),
        )?
        .transaction()
        .max_gas()
        .with_signer(user_account_id.clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();
    assert!(
        act_proposal_result.failures().is_empty(),
        "{:?}",
        act_proposal_result.failures()
    );

    // Verify the code of testdao.sputnik-dao.near matches the local build of sputnikdao2.wasm
    let upgraded_code = dao_contract.wasm().fetch_from(&sandbox_network).await?.data;
    assert_eq!(
        upgraded_code.code_base64,
        general_purpose::STANDARD.encode(sputnikdao2_wasm)
    );

    // Verify the DAO configuration remains the same
    let config: Value = dao_contract
        .call_function("get_config", ())?
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;
    assert_eq!(create_dao_args["config"], config);

    // Act on the transfer proposal
    let act_proposal_result = dao_contract
        .call_function(
            "act_proposal",
            json!({
                "id": transfer_proposal_id,
                "action": "VoteApprove",
                "proposal": {
                    "Transfer": {
                        "token_id": "",
                        "receiver_id": user_account_id,
                        "amount": "1"
                    },
                }
            }),
        )?
        .transaction()
        .max_gas()
        .with_signer(user_account_id.clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();
    assert!(
        act_proposal_result.failures().is_empty(),
        "{:?}",
        act_proposal_result.failures()
    );

    Ok(())
}
