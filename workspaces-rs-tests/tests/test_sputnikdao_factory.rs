use near_sdk::json_types::{Base58CryptoHash, Base64VecU8};
use near_units::{parse_gas, parse_near};
use sputnikdao2::{Config, VersionedPolicy};
use std::str::FromStr;
use workspaces::types::{KeyType, SecretKey};
use workspaces::AccountId;

#[tokio::test]
async fn test_factory() -> anyhow::Result<()> {
    // Create a sandbox environment.
    let worker = workspaces::sandbox().await?;
    println!("SPUTNIK FACTORY TESTS:\n\n");
    // Deploy Spunik DAO factory contract in sandbox
    println!("1. Can instantiate a new factory with default struct, \nincluding DAOs set:\n");
    let wasm = std::fs::read("../sputnikdao-factory2/res/sputnikdao_factory2.wasm")?;
    let dao_factory = worker
        .create_tla_and_deploy(
            AccountId::from_str("sputnik-factory.test.near")?,
            SecretKey::from_random(KeyType::ED25519),
            &wasm,
        )
        .await?
        .into_result()?;

    println!(
        "Creating an account to hold wasm file: Factory Contract Id: {:?}",
        dao_factory.id().as_str()
    );

    // Init daofactory contract
    println!("Instantiating new factory.");
    let init_daofactory = dao_factory
        .call("new")
        .gas(parse_gas!("42 Tgas") as u64)
        .transact()
        .await?
        .into_result()?;

    println!(
        "Instantiation completed. Outcome block hash: {:?}",
        init_daofactory.outcome().block_hash
    );

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

    println!("Getting the factory owner...");
    let get_owner: AccountId = dao_factory.view("get_owner").await?.json()?;
    println!("Factory owner: {:?}", get_owner.as_str());

    println!("Getting default code hash...");
    let get_default_code_hash: Base58CryptoHash =
        dao_factory.view("get_default_code_hash").await?.json()?;
    println!(
        "Code Hash: {:?}",
        serde_json::json!(get_default_code_hash).as_str().unwrap()
    );

    println!("Getting daos list...");
    let get_dao_list: Vec<AccountId> = dao_factory.view("get_dao_list").await?.json()?;
    for i in &get_dao_list {
        println!("DAO List: {i}");
    }

    println!("Getting code...");
    let get_code = dao_factory
        .view("get_code")
        .args_json(serde_json::json!({ "code_hash": get_default_code_hash, }))
        .await?
        .result;
    println!("Code len: {:?}.", get_code.len());

    Ok(())
}
