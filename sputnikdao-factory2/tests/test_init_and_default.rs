use near_sdk::{json_types::Base64VecU8, AccountId};
use near_units::parse_near;
use serde_json::json;
use sputnikdao2::{Config, VersionedPolicy};
use workspaces::{Contract, DevNetwork, Worker};

const FACTORY_WASM: &[u8] = include_bytes!("../res/sputnikdao_factory2.wasm");

async fn init(_worker: &Worker<impl DevNetwork>) -> anyhow::Result<Contract> {
    let worker = workspaces::sandbox().await?;
    let contract = worker.dev_deploy(FACTORY_WASM).await?;

    Ok(contract)
}
#[tokio::test]
async fn init_and_default() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let contract = init(&worker).await?;

    let res_new = contract
        .call("new")
        .max_gas()
        // .deposit(parse_near!("50 N")) Smart contract panicked:
        // Method new doesn't accept deposit.
        .transact()
        .await?;

    assert!(res_new.is_success());

    let config = Config {
        name: "testdao".to_string(),
        purpose: "to test".to_string(),
        metadata: Base64VecU8(vec![]),
    };
    let policy = VersionedPolicy::Default(vec![AccountId::new_unchecked("testing".to_string())]);
    let params = json!({ "config": config, "policy": policy })
        .to_string()
        .into_bytes();

    let res_create = contract
        .call("create")
        .args_json(json!({
                    "name":AccountId::new_unchecked("testdao".to_string()),
                    //testdao.dev-20221112014732-72260900390667
                    "args":Base64VecU8(params),
        }))
        .max_gas()
        .deposit(parse_near!("6 N"))
        .transact()
        .await?;

    assert!(res_create.is_success());

    let res_store = contract
        .call("store")
        .max_gas()
        .deposit(parse_near!("10 N"))
        .transact()
        .await?;

    assert!(res_store.is_success());

    let res_metadata = contract.view("get_contracts_metadata").await?;

    assert!(!res_metadata.result.is_empty());

    let res_re_init = contract.call("new").max_gas().transact().await?;
    // Smart contract panicked:
    // The contract has already been initialized
    assert!(res_re_init.is_failure());

    // let non_owner: AccountId = "testuser.near".parse().unwrap();

    // let res_not_owner = contract
    //     .as_account()
    //     .call(&non_owner, "new")
    //     .max_gas()
    //     .transact()
    //     .await?
    //     .into_result()?;
    // ######################################################################
    // Getting an error: Action #0: Can't complete the action because account
    // AccountId("testuser.near") doesn't exist
    // ######################################################################

    Ok(())
}
