use anyhow::Ok;
use near_sdk::{json_types::Base64VecU8, AccountId};
use near_units::parse_near;
use serde_json::json;
use sputnikdao2::{Config, VersionedPolicy};

const FACTORY_WASM: &[u8] = include_bytes!("../res/sputnikdao_factory2.wasm");

#[tokio::test]
async fn daos_creation() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let contract = worker.dev_deploy(FACTORY_WASM).await?;
    // let test_accoutnt = worker.dev_create_account().await?;

    let res_new = contract.call("new").max_gas().transact().await?;

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
                    "args":Base64VecU8(params),
        }))
        .max_gas()
        .deposit(parse_near!("6 N"))
        .transact()
        .await?;

    assert!(res_create.is_success());

    // let res_get_daos = contract.view("get_daos").await?;

    // assert!(!res_get_daos.result.is_empty());

    // will be updated after more research is dont on workspaces-rs v0.7.0

    Ok(())
}
