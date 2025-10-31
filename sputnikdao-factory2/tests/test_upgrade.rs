use near_api::types::{AccessKey, AccessKeyPermission};
use near_api::{AccountId, Contract, NearToken};
use near_sandbox::config::{DEFAULT_GENESIS_ACCOUNT, DEFAULT_GENESIS_ACCOUNT_PRIVATE_KEY};
use near_sdk::base64::{engine::general_purpose, Engine as _};
use near_sdk::env::sha256;
use near_sdk::json_types::Base58CryptoHash;
use near_sdk::serde_json::{json, Value};
use near_sdk::{AccountIdRef, CryptoHash};
use std::fs;
use std::str::FromStr;

#[tokio::test]
async fn test_upgrade() -> Result<(), Box<dyn std::error::Error>> {
    const SPUTNIKDAO_FACTORY_CONTRACT_ACCOUNT: &AccountIdRef =
        AccountIdRef::new_or_panic("sputnik-dao.near");

    // Import the mainnet sputnik-dao.near factory contract
    let sputnikdao_factory_contract =
        near_api::Contract(SPUTNIKDAO_FACTORY_CONTRACT_ACCOUNT.to_owned());

    let sandbox = near_sandbox::Sandbox::start_sandbox().await?;
    let sandbox_network =
        near_api::NetworkConfig::from_rpc_url("sandbox", sandbox.rpc_addr.parse()?);
    let signer = near_api::Signer::new(near_api::Signer::from_secret_key(
        DEFAULT_GENESIS_ACCOUNT_PRIVATE_KEY.parse()?,
    ))?;

    let user_account_id: AccountId = format!("some_account.{}", DEFAULT_GENESIS_ACCOUNT)
        .parse()
        .unwrap();
    near_api::Account::create_account(user_account_id.clone())
        .fund_myself(DEFAULT_GENESIS_ACCOUNT.to_owned(), NearToken::from_near(50))
        .public_key(signer.clone().get_public_key().await?)
        .unwrap()
        .with_signer(signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    let mut account = near_api::Account(sputnikdao_factory_contract.0.clone())
        .view()
        .fetch_from_mainnet()
        .await?
        .data;
    let wasm = sputnikdao_factory_contract
        .wasm()
        .fetch_from_mainnet()
        .await?
        .data;
    account.amount = NearToken::from_near(100);
    sandbox
        .patch_state(sputnikdao_factory_contract.0.clone())
        .account(account)
        .code(wasm.code_base64)
        .access_key(
            signer.get_public_key().await?.to_string(),
            AccessKey {
                nonce: 0.into(),
                permission: AccessKeyPermission::FullAccess,
            },
        )
        .send()
        .await
        .unwrap();

    // Initialize the sputnik-dao factory contract
    let init_sputnik_dao_factory_result = sputnikdao_factory_contract
        .call_function("new", ())
        .unwrap()
        .transaction()
        .max_gas()
        .with_signer(sputnikdao_factory_contract.0.clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?;
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
                "Group": [user_account_id.clone()],
            },
            "name": "Create Requests",
            "permissions": [
                "*:AddProposal"
            ],
            "vote_policy": {},
            },
            {
            "kind": {
                "Group": [user_account_id.clone()],
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
                "Group": [user_account_id.clone()],
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

    let create_result = sputnikdao_factory_contract
        .call_function(
            "create",
            json!({
                "name": dao_name,
                "args": general_purpose::STANDARD.encode(create_dao_args.to_string())
            }),
        )
        .unwrap()
        .transaction()
        .max_gas()
        .deposit(NearToken::from_near(6))
        .with_signer(user_account_id.clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?;

    assert!(create_result.is_success(), "{:?}", create_result.failures());

    let dao_account_id: AccountId = format!("{}.{}", dao_name, SPUTNIKDAO_FACTORY_CONTRACT_ACCOUNT)
        .parse()
        .unwrap();
    let dao_contract = near_api::Contract(dao_account_id.clone());

    // Verify the DAO configuration
    let config: Value = dao_contract
        .call_function("get_config", ())
        .unwrap()
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;
    assert_eq!(create_dao_args["config"], config);

    // Deploy the local build of the sputnik-dao factory contract
    let wasm = fs::read("./res/sputnikdao_factory2.wasm").expect("Unable to read contract wasm");
    let deploy_result = Contract::deploy(sputnikdao_factory_contract.0.clone())
        .use_code(wasm.to_vec())
        .without_init_call()
        .with_signer(signer.clone())
        .send_to(&sandbox_network)
        .await?;
    assert!(deploy_result.is_success());

    // Store the local build of sputnikdao2.wasm into sputnik-dao.near
    let sputnikdao2_wasm =
        fs::read("../sputnikdao2/res/sputnikdao2.wasm").expect("Unable to read sputnikdao2.wasm");
    let computed_hash = sha256(&sputnikdao2_wasm);
    let stored_contract_hash_string = sputnikdao_factory_contract
        .call_function_raw("store", sputnikdao2_wasm.clone())
        .transaction()
        .max_gas()
        .deposit(NearToken::from_near(20))
        .with_signer(sputnikdao_factory_contract.0.clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .json::<String>()
        .unwrap();

    let stored_contract_hash =
        Base58CryptoHash::from_str(stored_contract_hash_string.as_str()).unwrap();

    // Set the stored contract hash as the default code hash
    let set_default_code_hash_result = sputnikdao_factory_contract
        .call_function(
            "set_default_code_hash",
            json!({"code_hash": stored_contract_hash}),
        )
        .unwrap()
        .transaction()
        .with_signer(sputnikdao_factory_contract.0.clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?;
    assert!(
        set_default_code_hash_result.is_success(),
        "stored contract hash {:?}, failures: {:?}",
        stored_contract_hash,
        set_default_code_hash_result.failures()
    );

    // Verify the default code hash matches the computed hash
    let hash: Base58CryptoHash = sputnikdao_factory_contract
        .call_function("get_default_code_hash", ())
        .unwrap()
        .read_only()
        .fetch_from(&sandbox_network)
        .await?
        .data;

    assert_eq!(
        CryptoHash::from(hash).to_vec(),
        computed_hash,
        "Hashes do not match"
    );

    // Create a self-upgrade proposal
    let proposal_id = dao_contract
        .call_function(
            "add_proposal",
            json!({ "proposal": {
                "description": "proposal to test".to_string(),
                "kind": {"UpgradeSelf": {
                    "hash": stored_contract_hash
                }}
            }}),
        )
        .unwrap()
        .transaction()
        .max_gas()
        .deposit(NearToken::from_near(1))
        .with_signer(user_account_id.clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .json::<u64>()
        .unwrap();

    assert_eq!(0, proposal_id);

    // Create a transfer proposal to check after the upgrade
    let transfer_proposal_id = dao_contract
        .call_function(
            "add_proposal",
            json!({ "proposal": {
                "description": "a transfer proposal to check after upgrade",
                "kind": {
                    "Transfer": {
                        "token_id": "",
                        "receiver_id": user_account_id.clone(),
                        "amount": "1"
                    },
                },
            }}),
        )
        .unwrap()
        .transaction()
        .max_gas()
        .deposit(NearToken::from_near(1))
        .with_signer(user_account_id.clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .json::<u64>()
        .unwrap();

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
                }}
            }),
        )
        .unwrap()
        .transaction()
        .max_gas()
        .with_signer(user_account_id.clone(), signer.clone())
        .send_to(&sandbox_network)
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
    let upgraded_code = dao_contract.wasm().fetch_from(&sandbox_network).await?.data;
    assert_eq!(
        upgraded_code.code_base64,
        general_purpose::STANDARD.encode(sputnikdao2_wasm)
    );

    // Verify the DAO configuration remains the same
    let config: Value = dao_contract
        .call_function("get_config", ())
        .unwrap()
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
                        "receiver_id": user_account_id.clone(),
                        "amount": "1"
                    },
                }
            }),
        )
        .unwrap()
        .transaction()
        .max_gas()
        .with_signer(user_account_id.clone(), signer.clone())
        .send_to(&sandbox_network)
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
