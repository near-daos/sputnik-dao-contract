#![allow(unused_imports)]

use near_primitives::errors::{ActionError, ActionErrorKind};
use near_primitives::transaction::ExecutionOutcomeWithId;
use near_primitives::views::ExecutionOutcomeWithIdView;
use near_sdk::json_types::{self, Base58CryptoHash, Base64VecU8};
use near_units::{parse_gas, parse_near};
use sputnikdao2::{Config, VersionedPolicy};
use std::f32::consts::E;
use std::str::FromStr;
use workspaces::operations::TransactionStatus;
use workspaces::types::{KeyType, SecretKey};
use workspaces::AccountId;

const SPUTNIK_FACTORY_WASM: &str = "../sputnikdao-factory2/res/sputnikdao_factory2.wasm";

#[tokio::test]
async fn test_factory() -> anyhow::Result<()> {
    println!("SPUTNIK FACTORY TESTS:\n");

    // Create a sandbox environment.
    let worker = workspaces::sandbox().await?;

    let wasm = std::fs::read(SPUTNIK_FACTORY_WASM)?;

    println!("Testing: Can instantiate a new factory with default struct.");

    // Deploy Spunik DAO factory contract in sandbox
    let dao_factory = worker
        .create_tla_and_deploy(
            AccountId::from_str("sputnik-factory.test.near")?,
            SecretKey::from_random(KeyType::ED25519),
            &wasm,
        )
        .await?
        .into_result()?;
    assert_eq!(
        dao_factory.id().as_str(),
        "sputnik-factory.test.near",
        "Failed to create an account!"
    );
    println!(
        "Created an account to hold wasm file: {:?}",
        dao_factory.id().as_str()
    );

    // Init daofactory contract
    println!("Initiating a SputnikDAO Factory.");
    println!("Testing: Only factory owner can call \"new\".");
    let some_user = worker
        .create_tla(
            AccountId::from_str("some-user.test.near")?,
            SecretKey::from_random(KeyType::ED25519),
        )
        .await?
        .into_result()?;

    let factory_wrong_owner = some_user
        .call(&dao_factory.id(), "new")
        .gas(parse_gas!("42 Tgas") as u64)
        .transact()
        .await?;
    assert!(
        factory_wrong_owner.is_failure(),
        "Factory initiated succesfully with non owner calling \"new\"!!"
    );

    //-------------------------------------------------------------------------

    let factory_init = dao_factory
        .call("new")
        .gas(parse_gas!("42 Tgas") as u64)
        .transact()
        .await?;
    assert!(factory_init.is_success(), "Factory failed to initiate!");
    println!(
        "Factory successfuly initiatet? [ {:?} ]",
        factory_init.is_success()
    );

    //  Re-init daofactory contract
    println!("Testing: Does not allow re-init of a factory.\nRe-initiating a SputnikDAO Factory.");
    let factory_re_init = dao_factory
        .call("new")
        .gas(parse_gas!("42 Tgas") as u64)
        .transact()
        .await?;
    assert!(
        factory_re_init.is_failure(),
        "Factory re-initiated succesfully???"
    );

    // "Non factory owner was able to call \"new\"!!");

    // Define parameters of new dao:

    // Configure name, purpose, and initial council members of the DAO and convert the arguments in base64
    let config = Config {
        name: "sputnik-dao".to_string(),
        purpose: "Sputnik internal test DAO".to_string(),
        metadata: Base64VecU8(vec![]),
    };
    let policy = VersionedPolicy::Default(vec![]);
    let params = serde_json::json!({ "config": config, "policy": policy, })
        .to_string()
        .into_bytes();

    // Create a new DAO
    println!("Creating new DAO");
    let create_new_dao = dao_factory
        .call("create")
        .args_json(serde_json::json!({
            "name": "awesome",
            "args":Base64VecU8(params),
        }))
        .deposit(parse_near!("6 N"))
        .gas(parse_gas!("150 Tgas") as u64)
        .transact()
        .await?
        .into_result()?;
    println!(
        "DAO successfully created. Outcome block hash: {:?}",
        create_new_dao.outcome().block_hash
    );
    println!("Getting default code hash...");
    let default_code_hash: Base58CryptoHash =
        dao_factory.view("get_default_code_hash").await?.json()?;
    println!(
        "Code Hash: {:?}",
        serde_json::json!(default_code_hash).as_str().unwrap()
    );

    println!("Getting daos list...");
    let get_dao_list: Vec<AccountId> = dao_factory.view("get_dao_list").await?.json()?;
    for i in &get_dao_list {
        println!("DAO List: {i}");
    }

    println!("Getting the factory owner...");
    let factory_owner: AccountId = dao_factory.view("get_owner").await?.json()?;
    println!("Factory owner: {:?}", factory_owner.as_str());

    println!("Getting code...");
    let get_code = dao_factory
        .view("get_code")
        .args_json(serde_json::json!({ "code_hash": default_code_hash, }))
        .await?
        .result;
    println!("Code len: {:?}.", get_code.len());

    Ok(())
}
