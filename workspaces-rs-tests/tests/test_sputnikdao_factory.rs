use near_sdk::json_types::Base64VecU8;
use near_units::{parse_gas, parse_near};
use sputnikdao2::{Config, VersionedPolicy};
use std::str::FromStr;
use near_workspaces as workspaces;
use workspaces::types::{KeyType, SecretKey};
use workspaces::AccountId;
use workspaces::types::{KeyType, SecretKey};
use workspaces::AccountId;

//-----------------------------------------------------------------------------

const FACTORY: &str = "../sputnikdao-factory2/res/sputnikdao_factory2.wasm";

//-----------------------------------------------------------------------------

#[tokio::test]
async fn test_factory() -> anyhow::Result<()> {
    println!("Testing: SPUTNIK FACTORY\n");

    // Create a sandbox environment.
    let worker = workspaces::sandbox().await?;
    let wasm = std::fs::read(FACTORY)?;

    //-------------------------------------------------------------------------

    println!("\nTesting: Can instantiate a new factory with default struct.");

    // Deploy Spunik DAO factory contract in sandbox.
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
        "❌ Failed to create an account!"
    );
    println!(
        "✅ Created an account to hold wasm file: {:?}",
        dao_factory.id().as_str()
    );

    //-------------------------------------------------------------------------

    // Init daofactory contract with a wrong owner.
    println!("\nTesting: Only factory owner can call \"new\".");
    println!("Initiating a SputnikDAO Factory with a wrong owner.");
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
        "❌ Factory initiated succesfully with non owner calling \"new\"!!"
    );
    println!("✅ Must be owner Error");

    //-------------------------------------------------------------------------

    // Init daofactory contract with a correct owner now.
    println!("\nTesting: Can instantiate a new factory.");
    println!("Initiating a SputnikDAO Factory");
    let factory_init = dao_factory
        .call("new")
        .gas(parse_gas!("42 Tgas") as u64)
        .transact()
        .await?;
    assert!(factory_init.is_success(), "❌ Factory failed to initiate!");
    println!("✅ Factory successfuly initiatet.");

    //-------------------------------------------------------------------------

    // Checking the factory owner.
    println!("\nTesting: get_owner Returns the account that owns the factory");
    let owner: AccountId = dao_factory.view("get_owner").await?.json()?;
    assert_eq!(dao_factory.id().as_str(), owner.as_str(), "❌ Error");
    println!("✅ Factory owner: {:?}", owner.as_str());

    //-------------------------------------------------------------------------

    // Returns empty array for new factory
    println!("\nTesting: get_dao_list Returns empty array for new factory");
    let get_dao_list_new_factory: Vec<AccountId> =
        dao_factory.view("get_dao_list").await?.json()?;
    assert!(
        get_dao_list_new_factory.is_empty(),
        "❌ New factory returned a non empty DAO list."
    );
    println!("✅ DAO list: {:?}", get_dao_list_new_factory);

    //-------------------------------------------------------------------------

    //  Re-init daofactory contract.
    println!("\nTesting: Does not allow re-init of a factory.");
    println!("Trying to re-initiate a SputnikDAO Factory.");
    let factory_re_init = dao_factory
        .call("new")
        .gas(parse_gas!("42 Tgas") as u64)
        .transact()
        .await?;
    assert!(
        factory_re_init.is_failure(),
        "❌ Factory re-initiated succesfully!!!"
    );
    println!("✅ The contract has already been initialized Error");

    //-------------------------------------------------------------------------

    // Define parameters of new dao:

    // Configure name, purpose, and initial council members of the DAO and
    // convert the arguments in base64.
    let config = Config {
        name: "sputnik-dao".to_string(),
        purpose: "Sputnik internal test DAO".to_string(),
        metadata: Base64VecU8(vec![]),
    };
    let policy = VersionedPolicy::Default(vec![]);
    let params = serde_json::json!({ "config": config, "policy": policy, })
        .to_string()
        .into_bytes();

    //-------------------------------------------------------------------------

    // Creating a dao with "some-user" account.
    println!("\nTesting: Allows any account to call create method.");
    let create_new_dao = some_user
        .call(&dao_factory.id(), "create")
        .args_json(serde_json::json!({
            "name": "test",
            "args":Base64VecU8(params.clone()),
        }))
        .deposit(parse_near!("6 N"))
        .gas(parse_gas!("150 Tgas") as u64)
        .transact()
        .await?;
    assert!(
        create_new_dao.is_success(),
        "❌ Failed to create a DAO using \"some_user\" as an account."
    );
    println!(
        "✅ DAO successfully created using {:?} as an account",
        some_user.id().as_str()
    );

    //-------------------------------------------------------------------------

    // Creating a dao with factory owner account.
    println!("\nTesting: Allows any account to call create method.");
    let create_new_dao = dao_factory
        .call("create")
        .args_json(serde_json::json!({
            "name": "awesome",
            "args":Base64VecU8(params.clone()),
        }))
        .deposit(parse_near!("6 N"))
        .gas(parse_gas!("150 Tgas") as u64)
        .transact()
        .await?;
    assert!(
        create_new_dao.is_success(),
        "❌ Failed to create a DAO using factory owner as an account."
    );
    println!(
        "✅ DAO successfully created using {:?} as an account",
        dao_factory.id().as_str()
    );

    //-------------------------------------------------------------------------

    // Checking for daos to see if dao is in a DAO list.
    println!("\nTesting: Created DAO becomes a sub-account of the factory.");
    let get_dao_list_dao_in_a_list: Vec<AccountId> =
        dao_factory.view("get_dao_list").await?.json()?;
    assert_eq!(
        get_dao_list_dao_in_a_list.last().unwrap().as_str(),
        "awesome.sputnik-factory.test.near",
        "❌ Dao not sub-account of the factory"
    );
    println!(
        "✅ DAO exists in the list: {:?}",
        get_dao_list_dao_in_a_list
    );

    //-------------------------------------------------------------------------

    // Creating a dao with same dao name.
    println!("\nTesting: Fails if the DAO name exists.");
    let create_new_dao_same_name = dao_factory
        .call("create")
        .args_json(serde_json::json!({
            "name": "awesome",
            "args":Base64VecU8(params.clone()),
        }))
        .deposit(parse_near!("6 N"))
        .gas(parse_gas!("150 Tgas") as u64)
        .transact()
        .await?;
    assert!(
        create_new_dao_same_name.is_failure(),
        "❌ Succesfully created a dao with name that already exists."
    );
    println!("✅ AccountAlreadyExists Error");

    //-------------------------------------------------------------------------

    // Creating a dao with invalid account ID.
    println!("\nTesting: Fails if the DAO name is not a valid account ID.");
    let create_new_dao_wrong_name = dao_factory
        .call("create")
        .args_json(serde_json::json!({
            "name": "//",
            "args":Base64VecU8(params.clone()),
        }))
        .deposit(parse_near!("6 N"))
        .gas(parse_gas!("150 Tgas") as u64)
        .transact()
        .await?;
    assert!(
        create_new_dao_wrong_name.is_failure(),
        "❌ Succesfully created a dao with invalid acconut ID."
    );
    println!("✅ The account ID is invalid Error");

    //-------------------------------------------------------------------------

    // println!("Getting default code hash...");
    // let default_code_hash: Base58CryptoHash =
    //     dao_factory.view("get_default_code_hash").await?.json()?;
    // println!(
    //     "Code Hash: {:?}",
    //     serde_json::json!(default_code_hash).as_str().unwrap()
    // );

    // //-------------------------------------------------------------------------

    // println!("Getting the factory owner...");
    // let factory_owner: AccountId = dao_factory.view("get_owner").await?.json()?;
    // println!("Factory owner: {:?}", factory_owner.as_str());

    // //-------------------------------------------------------------------------

    // println!("Getting code...");
    // let get_code = dao_factory
    //     .view("get_code")
    //     .args_json(serde_json::json!({ "code_hash": default_code_hash, }))
    //     .await?
    //     .result;
    // println!("Code len: {:?}.", get_code.len());

    Ok(())
}
