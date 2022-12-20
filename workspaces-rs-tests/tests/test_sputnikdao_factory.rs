use near_sdk::json_types::{Base58CryptoHash, Base64VecU8};
use near_sdk::AccountId as AccID;
use near_units::parse_near;
use sputnikdao2::{Config, VersionedPolicy};
use std::str::FromStr;
use workspaces::types::{KeyType, SecretKey};
use workspaces::AccountId;



#[tokio::test]
async fn test_factory() -> anyhow::Result<()> {
    // Create a sandbox environment.
    let worker = workspaces::sandbox().await?;

    // Deploy Spunik DAO factory contract in sandbox
    println!("Deploying Spunik DAO factory contract");
    let wasm = std::fs::read("../sputnikdao-factory2/res/sputnikdao_factory2.wasm")?;
    let dao_factory = worker
        .create_tla_and_deploy(
            AccountId::from_str("dao-factory.test.near")?,
            SecretKey::from_random(KeyType::ED25519),
            &wasm,
        )
        .await?
        .unwrap();

    println!("Contract Id: {:?}", dao_factory.id());

    // Init daofactory contract
    println!("Initializing daofactory contract");
    let init_daofactory = dao_factory
        .call("new")
        .max_gas()
        .transact()
        .await?
        .borsh()?;
    println!("Initialization complete. {:?}", init_daofactory);

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
            "name": "createdao",
            "args":Base64VecU8(params),
        }))
        .deposit(parse_near!("6 N"))
        .max_gas()
        .transact()
        .await?
        .borsh()?;
    println!("DAO successfully created. {:?}", create_new_dao);

    println!("Getting the factory owner...");
    let get_owner = AccID::new_unchecked(dao_factory.view("get_owner").await?.json()?);
    println!("Factory owner: {:?}", get_owner);

    println!("Getting default code hash...");
    let get_default_code_hash: Base58CryptoHash =
        dao_factory.view("get_default_code_hash").await?.json()?;
    println!("Code Hash: {:?}", get_default_code_hash);

    println!("Getting daos list...");
    let get_dao_list: Vec<AccountId> = dao_factory.view("get_dao_list").await?.json()?;
    println!("List of DAOs {:?}", get_dao_list);

    println!("Getting code...");
    let get_code = dao_factory
        .view("get_code")
        .args_json(serde_json::json!({ "code_hash": get_default_code_hash, }))
        .await?
        .result;
    println!("Code lenght: {:?}", get_code.len());

    Ok(())
}
