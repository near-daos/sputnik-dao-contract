use near_sdk::serde_json::{json, Value};
use near_sdk::{
    base64::{engine::general_purpose, Engine as _},
    AccountIdRef,
};
use near_workspaces::types::NearToken;
use near_workspaces::AccountId;
use std::fs;

#[tokio::test]
async fn test_global_contract_deployment() -> Result<(), Box<dyn std::error::Error>> {
    const SPUTNIKDAO_FACTORY_CONTRACT_ACCOUNT: &str = "sputnik-dao.near";

    let mainnet = near_workspaces::custom("https://rpc.mainnet.fastnear.com").await?;
    let sputnikdao_factory_contract_id: AccountId =
        AccountIdRef::new_or_panic(SPUTNIKDAO_FACTORY_CONTRACT_ACCOUNT).into();

    let worker = near_workspaces::sandbox_with_version("2.8.0").await?;

    // Import and deploy the sputnik-dao factory contract
    let sputnik_dao_factory = worker
        .import_contract(&sputnikdao_factory_contract_id, &mainnet)
        .initial_balance(NearToken::from_near(100))
        .transact()
        .await?;

    let wasm = fs::read("./res/sputnikdao_factory2.wasm").expect("Unable to read contract wasm");

    let deploy_result = sputnik_dao_factory
        .as_account()
        .deploy(wasm.as_slice())
        .await?;
    assert!(deploy_result.is_success());

    // Initialize the factory contract
    let init_sputnik_dao_factory_result = sputnik_dao_factory
        .call("new")
        .max_gas()
        .transact()
        .await?;
    if init_sputnik_dao_factory_result.is_failure() {
        panic!(
            "Error initializing sputnik-dao contract: {:?}",
            String::from_utf8(init_sputnik_dao_factory_result.raw_bytes().unwrap())
        );
    }
    assert!(init_sputnik_dao_factory_result.is_success());

    // Deploy the DAO contract as a global contract
    let deploy_dao_global_result = sputnik_dao_factory
        .call("deploy_dao_global_contract")
        .max_gas()
        .transact()
        .await?;

    assert!(
        deploy_dao_global_result.is_success(),
        "Failed to deploy global DAO contract: {:?}",
        deploy_dao_global_result.failures()
    );

    // Create test DAO args
    let dao_name = "testdao-global";
    let create_dao_args = json!({
        "config": {
            "name": dao_name,
            "purpose": "testing global contract deployment",
            "metadata": "",
        },
        "policy": {
            "roles": [
                {
                    "kind": {
                        "Group": ["acc3.near", "acc2.near", "acc1.near"],
                    },
                    "name": "Create Requests",
                    "permissions": [
                        "call:AddProposal",
                        "transfer:AddProposal",
                        "config:Finalize",
                    ],
                    "vote_policy": {},
                },
                {
                    "kind": {
                        "Group": ["acc1.near"],
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
                        "Group": ["acc1.near", "acc2.near"],
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
            "proposal_bond": "100000000000000000000000",
            "proposal_period": "604800000000000",
            "bounty_bond": "100000000000000000000000",
            "bounty_forgiveness_period": "604800000000000",
        },
    });

    let user_account = worker.dev_create_account().await?;

    // Track user balance before creating DAO
    let balance_before = user_account.view_account().await?.balance;

    // Track factory balance before creating DAO
    let factory_balance_before = sputnik_dao_factory.view_account().await?.balance;

    // Create DAO using global contract with 1 NEAR deposit (reduced from 6 NEAR)
    let create_result = user_account
        .call(&sputnikdao_factory_contract_id, "create_global_contract")
        .args_json(json!({
            "name": dao_name,
            "args": general_purpose::STANDARD.encode(create_dao_args.to_string())
        }))
        .max_gas()
        .deposit(NearToken::from_near(1))
        .transact()
        .await?;

    assert!(
        create_result.is_success(),
        "Failed to create DAO with global contract: {:?}",
        create_result.failures()
    );

    // Verify the DAO was created correctly
    let dao_account_id: AccountId = format!("{}.{}", dao_name, SPUTNIKDAO_FACTORY_CONTRACT_ACCOUNT)
        .parse()
        .unwrap();

    let get_config_result = worker.view(&dao_account_id, "get_config").await?;

    let config: Value = get_config_result.json().unwrap();
    assert_eq!(create_dao_args["config"], config);

    // Print DAO account balance information
    let dao_account = worker.view_account(&dao_account_id).await?;
    let total_balance = dao_account.balance;
    let storage_usage = dao_account.storage_usage;

    // Storage cost: 1 byte = 10^19 yoctoNEAR (0.00001 NEAR)
    const STORAGE_PRICE_PER_BYTE: u128 = 10_000_000_000_000_000_000; // 10^19 yoctoNEAR
    let storage_cost = storage_usage as u128 * STORAGE_PRICE_PER_BYTE;
    let available_balance = total_balance.as_yoctonear().saturating_sub(storage_cost);

    println!("\n=== DAO Account Balance Information ===");
    println!("DAO Account ID: {}", dao_account_id);
    println!("Total Balance: {} NEAR ({} yoctoNEAR)",
        total_balance.as_near(),
        total_balance.as_yoctonear()
    );
    println!("Storage Usage: {} bytes", storage_usage);
    println!("Storage Reserved: {} NEAR ({} yoctoNEAR)",
        NearToken::from_yoctonear(storage_cost).as_near(),
        storage_cost
    );
    println!("Available Balance: {} NEAR ({} yoctoNEAR)",
        NearToken::from_yoctonear(available_balance).as_near(),
        available_balance
    );

    // The factory deducts 0.01 NEAR gas buffer, so DAO receives 0.99 NEAR + gas refunds
    let expected_transfer = NearToken::from_millinear(990); // 0.99 NEAR
    let gas_refund = total_balance.as_yoctonear().saturating_sub(expected_transfer.as_yoctonear());
    println!("Expected Transfer (after gas buffer): {} NEAR", expected_transfer.as_near());
    println!("Gas Refund from Initialization: {} NEAR ({} yoctoNEAR)",
        NearToken::from_yoctonear(gas_refund).as_near(),
        gas_refund
    );
    println!("======================================\n");

    // Assert storage usage is less than 2KB
    assert!(
        storage_usage < 2048,
        "Storage usage should be less than 2KB, but got {} bytes",
        storage_usage
    );

    // Assert storage balance is less than 0.2 NEAR
    assert!(
        storage_cost < NearToken::from_millinear(200).as_yoctonear(),
        "Storage balance should be less than 0.2 NEAR, but got {} NEAR",
        NearToken::from_yoctonear(storage_cost).as_near()
    );

    // Assert available balance is greater than 0.78 NEAR but less than 1 NEAR
    // (0.99 NEAR transfer + gas refunds - 0.2 NEAR max storage = ~0.79+ NEAR available)
    assert!(
        available_balance > NearToken::from_millinear(780).as_yoctonear(),
        "Available balance should be greater than 0.78 NEAR, but got {} NEAR",
        NearToken::from_yoctonear(available_balance).as_near()
    );
    assert!(
        available_balance < NearToken::from_near(1).as_yoctonear(),
        "Available balance should be less than 1 NEAR, but got {} NEAR",
        NearToken::from_yoctonear(available_balance).as_near()
    );

    // Assert DAO total balance reflects the 0.99 NEAR transfer + gas refunds
    // The factory deducts 0.01 NEAR gas buffer, so DAO receives 0.99 NEAR + gas refunds
    // Gas refunds should bring the total close to 1 NEAR
    assert!(
        total_balance.as_yoctonear() >= NearToken::from_millinear(980).as_yoctonear(),
        "DAO total balance should be at least 0.98 NEAR (0.99 NEAR - small margin), but got {} NEAR",
        total_balance.as_near()
    );
    assert!(
        total_balance.as_yoctonear() <= NearToken::from_near(1).as_yoctonear(),
        "DAO total balance should not exceed 1 NEAR (0.99 NEAR + gas refunds), but got {} NEAR",
        total_balance.as_near()
    );

    // Verify factory balance increased due to gas rewards
    let factory_balance_after = sputnik_dao_factory.view_account().await?.balance;
    assert!(
        factory_balance_after.as_yoctonear() >= factory_balance_before.as_yoctonear(),
        "Factory balance should not decrease (should increase due to gas rewards). Before: {} NEAR, After: {} NEAR",
        factory_balance_before.as_near(),
        factory_balance_after.as_near()
    );
    println!(
        "✓ Factory balance increased from {} NEAR to {} NEAR (gas rewards: {} NEAR)",
        factory_balance_before.as_near(),
        factory_balance_after.as_near(),
        NearToken::from_yoctonear(
            factory_balance_after.as_yoctonear() - factory_balance_before.as_yoctonear()
        ).as_near()
    );

    // Verify the balance change - should be approximately 1 NEAR plus gas
    let balance_after = user_account.view_account().await?.balance;
    let balance_diff = balance_before.as_yoctonear() - balance_after.as_yoctonear();

    // Assert that at least 1 NEAR was spent (could be slightly more due to gas)
    assert!(
        balance_diff >= NearToken::from_near(1).as_yoctonear(),
        "Expected at least 1 NEAR to be spent, but only {} yoctoNEAR was spent",
        balance_diff
    );

    // Assert that less than 2 NEAR was spent (to verify it's not the old 6 NEAR cost)
    assert!(
        balance_diff < NearToken::from_near(2).as_yoctonear(),
        "Expected less than 2 NEAR to be spent, but {} yoctoNEAR was spent",
        balance_diff
    );

    println!(
        "✓ DAO created successfully with global contract using {} NEAR",
        NearToken::from_yoctonear(balance_diff).as_near()
    );

    
    Ok(())
}

