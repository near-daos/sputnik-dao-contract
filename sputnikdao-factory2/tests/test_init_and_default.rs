// macro allowing us to convert args into JSON bytes to be read by the contract.
use near_sdk::serde_json::json;

// macro allowing us to convert human readable units to workspace units.
use near_units::{parse_gas, parse_near};

// Additional convenient imports that allows workspaces to function readily.
use near_workspaces::prelude::*;

const FACTORY_WASM_FILE: &str = "./res/sputnikdao_factory2.wasm";

// const SPUTNIKDAO_WASM_FILE: &str = "../sputnikdao2/res/sputnikdao2.wasm";

#[tokio::test]
async fn deploy_new_factory() -> anyhow::Result<()> {
    let worker = near_workspaces::sandbox().await?;
    let wasm = std::fs::read(FACTORY_WASM_FILE)?;
    let contract = worker.dev_deploy(&wasm).await?;
    let account = contract.as_account();
    

//Initializes factory for the first time
    println!("Initializeing factory.");
    let _res = account
        .call(&worker, contract.id(),"new")
        .args_json(json!({"accountId":account.id()}))?
        .gas(parse_gas!("100 Tgas") as u64)
        .transact()
        .await?;
    println!("Factory initialized. AccountId: {}", account.id());

// Creating a DAO using the factory.
    println!("Creating a DAO");
    let _res = account
        .call(&worker, contract.id(),"create")
        .args_json(json!({"args":{"name":"genisis", "accountId":account.id()}}))?
        .deposit(parse_near!("50 N"))
        .gas(parse_gas!("200 Tgas") as u64)
        .transact()
        .await?;
        println!("DAO created.");
        Ok(())
}

//Fails to initialize factory a secont time with message:
//"Smart contract panicked: The contract has already been initialized"
    // println!("Initialize a factory the second time.");
    // let _res = account
    //     .call(&worker, account.id(),"new")
    //     .args_json(json!({"accountId":account.id()}))?
    //     .gas(parse_gas!("100 Tgas") as u64)
    //     .transact()
    //     .await?;
    // Ok(())