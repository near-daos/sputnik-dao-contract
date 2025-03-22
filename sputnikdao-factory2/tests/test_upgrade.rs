use near_sdk::base64::{engine::general_purpose, Engine as _};
use near_sdk::env::sha256;
use near_sdk::json_types::Base58CryptoHash;
use near_sdk::serde_json::{json, Value};
use near_sdk::{AccountIdRef, CryptoHash};
use near_workspaces::types::NearToken;
use near_workspaces::{AccountId, Contract};
use std::fs;
use std::str::FromStr;

#[tokio::test]
async fn test_upgrade() -> Result<(), Box<dyn std::error::Error>> {
    const SPUTNIKDAO_FACTORY_CONTRACT_ACCOUNT: &str = "sputnik-dao.near";

    // Import the mainnet sputnik-dao.near factory contract
    let mainnet = near_workspaces::mainnet().await?;
    let sputnikdao_factory_contract_id: AccountId =
        AccountIdRef::new_or_panic(SPUTNIKDAO_FACTORY_CONTRACT_ACCOUNT).into();

    let worker = near_workspaces::sandbox().await?;
    let user_account = worker.dev_create_account().await?;

    // Deploy the sputnik-dao.near factory contract to the sandbox
    let sputnik_dao_factory = worker
        .import_contract(&sputnikdao_factory_contract_id, &mainnet)
        .initial_balance(NearToken::from_near(100))
        .transact()
        .await?;

    // Initialize the sputnik-dao factory contract
    let init_sputnik_dao_factory_result =
        sputnik_dao_factory.call("new").max_gas().transact().await?;
    if init_sputnik_dao_factory_result.is_failure() {
        panic!(
            "Error initializing sputnik-dao contract: {:?}",
            String::from_utf8(init_sputnik_dao_factory_result.raw_bytes().unwrap())
        );
    }
    assert!(init_sputnik_dao_factory_result.is_success());

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
                "Group": [user_account.id()],
            },
            "name": "Create Requests",
            "permissions": [
                "*:AddProposal"
            ],
            "vote_policy": {},
            },
            {
            "kind": {
                "Group": [user_account.id()],
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
                "Group": [user_account.id()],
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

    let create_result = user_account
        .call(&sputnikdao_factory_contract_id, "create")
        .args_json(json!({
            "name": dao_name,
            "args": general_purpose::STANDARD.encode(create_dao_args.to_string())
        }))
        .max_gas()
        .deposit(NearToken::from_near(6))
        .transact()
        .await?;

    assert!(create_result.is_success(), "{:?}", create_result.failures());

    let dao_account_id: AccountId = format!("{}.{}", dao_name, SPUTNIKDAO_FACTORY_CONTRACT_ACCOUNT)
        .parse()
        .unwrap();
    let dao_contract = Contract::from_secret_key(
        dao_account_id.clone(),
        user_account.secret_key().clone(),
        &worker,
    );

    // Verify the DAO configuration
    let get_config_result = worker.view(&dao_account_id, "get_config").await?;
    let config: Value = get_config_result.json().unwrap();
    assert_eq!(create_dao_args["config"], config);

    // Deploy the local build of the sputnik-dao factory contract
    let wasm = fs::read("./res/sputnikdao_factory2.wasm").expect("Unable to read contract wasm");
    let deploy_result = sputnik_dao_factory
        .as_account()
        .deploy(wasm.as_slice())
        .await?;
    assert!(deploy_result.is_success());

    // Store the local build of sputnikdao2.wasm into sputnik-dao.near
    let sputnikdao2_wasm =
        fs::read("../sputnikdao2/res/sputnikdao2.wasm").expect("Unable to read sputnikdao2.wasm");
    let computed_hash = sha256(&sputnikdao2_wasm);
    let stored_contract_hash_string = sputnik_dao_factory
        .call("store")
        .args(sputnikdao2_wasm.clone())
        .max_gas()
        .deposit(NearToken::from_near(20))
        .transact()
        .await?
        .json::<String>()
        .unwrap();

    let stored_contract_hash =
        Base58CryptoHash::from_str(stored_contract_hash_string.as_str()).unwrap();

    // Set the stored contract hash as the default code hash
    let set_default_code_hash_result = sputnik_dao_factory
        .call("set_default_code_hash")
        .args_json(json!({"code_hash": stored_contract_hash}))
        .transact()
        .await?;
    assert!(
        set_default_code_hash_result.is_success(),
        "stored contract hash {:?}, failures: {:?}",
        stored_contract_hash,
        set_default_code_hash_result.failures()
    );

    // Verify the default code hash matches the computed hash
    let hash = sputnik_dao_factory
        .view("get_default_code_hash")
        .await?
        .json::<Base58CryptoHash>()
        .unwrap();

    assert_eq!(
        CryptoHash::from(hash).to_vec(),
        computed_hash,
        "Hashes do not match"
    );

    // Create a self-upgrade proposal
    let proposal_id = user_account
        .call(dao_contract.id(), "add_proposal")
        .args_json(json!({ "proposal": {
            "description": "proposal to test".to_string(),
            "kind": {"UpgradeSelf": {
                "hash": stored_contract_hash
            }}
        }}))
        .deposit(NearToken::from_near(1))
        .transact()
        .await?
        .unwrap()
        .json::<u64>()
        .unwrap();

    assert_eq!(0, proposal_id);

    // Create a transfer proposal to check after the upgrade
    let transfer_proposal_id = user_account
        .call(dao_contract.id(), "add_proposal")
        .args_json(json!({ "proposal": {
            "description": "a transfer proposal to check after upgrade",
            "kind": {
                "Transfer": {
                    "token_id": "",
                    "receiver_id": user_account.id(),
                    "amount": "1"
                },
            },
        }}))
        .deposit(NearToken::from_near(1))
        .transact()
        .await?
        .unwrap()
        .json::<u64>()
        .unwrap();

    // Act on the self-upgrade proposal
    let act_proposal_result = user_account
        .call(dao_contract.id(), "act_proposal")
        .args_json(json!({
            "id": 0,
            "action": "VoteApprove",
            "proposal": {
                "UpgradeSelf": {
                "hash": stored_contract_hash
            }}
        }))
        .max_gas()
        .transact()
        .await?;
    assert!(
        act_proposal_result.is_success(),
        "{:?}",
        act_proposal_result.failures()
    );
    assert_eq!(
        0,
        act_proposal_result.failures().len(),
        "{:?}",
        act_proposal_result.failures()
    );

    // Verify the code of testdao.sputnik-dao.near matches the local build of sputnikdao2.wasm
    let upgraded_code = dao_contract.view_code().await?;
    assert_eq!(upgraded_code, sputnikdao2_wasm);

    // Verify the DAO configuration remains the same
    let get_config_result = worker.view(&dao_account_id, "get_config").await?;
    let config: Value = get_config_result.json().unwrap();
    assert_eq!(create_dao_args["config"], config);

    // Act on the transfer proposal
    let act_proposal_result = user_account
        .call(dao_contract.id(), "act_proposal")
        .args_json(json!({
            "id": transfer_proposal_id,
            "action": "VoteApprove",
            "proposal": {
                "Transfer": {
                    "token_id": "",
                    "receiver_id": user_account.id(),
                    "amount": "1"
                },
            }
        }))
        .max_gas()
        .transact()
        .await?;

    assert!(
        act_proposal_result.is_success(),
        "{:?}",
        act_proposal_result.failures()
    );
    assert_eq!(
        0,
        act_proposal_result.failures().len(),
        "{:?}",
        act_proposal_result.failures()
    );

    Ok(())
}
