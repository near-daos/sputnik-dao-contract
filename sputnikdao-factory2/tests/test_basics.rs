use near_sdk::base64;
use near_sdk::serde_json::{json, Value};
use near_workspaces::types::NearToken;
use near_workspaces::{AccountId, Contract};
use std::fs;

#[tokio::test]
async fn test_factory() -> Result<(), Box<dyn std::error::Error>> {
    const SPUTNIKDAO_FACTORY_CONTRACT_ACCOUNT: &str = "sputnik-dao.near";

    let mainnet = near_workspaces::mainnet().await?;
    let sputnikdao_factory_contract_id: AccountId = SPUTNIKDAO_FACTORY_CONTRACT_ACCOUNT.parse()?;

    let worker = near_workspaces::sandbox().await?;

    let sputnik_dao_factory = worker
        .import_contract(&sputnikdao_factory_contract_id, &mainnet)
        .initial_balance(NearToken::from_near(50))
        .transact()
        .await?;

    let wasm = fs::read("./res/sputnikdao_factory2.wasm").expect("Unable to read contract wasm");

    let deploy_result = sputnik_dao_factory
        .as_account()
        .deploy(wasm.as_slice())
        .await?;
    assert!(deploy_result.is_success());

    let init_sputnik_dao_factory_result =
        sputnik_dao_factory.call("new").max_gas().transact().await?;
    if init_sputnik_dao_factory_result.is_failure() {
        panic!(
            "Error initializing sputnik-dao contract: {:?}",
            String::from_utf8(init_sputnik_dao_factory_result.raw_bytes().unwrap())
        );
    }
    assert!(init_sputnik_dao_factory_result.is_success());

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
    let account_details_before = user_account.view_account().await?;

    let create_result = user_account
        .call(&sputnikdao_factory_contract_id, "create")
        .args_json(json!({
            "name": dao_name,
            "args": base64::encode(create_dao_args.to_string())
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

    let get_config_result = worker.view(&dao_account_id, "get_config").await?;

    let config: Value = get_config_result.json().unwrap();
    assert_eq!(create_dao_args["config"], config);

    Ok(())
}
