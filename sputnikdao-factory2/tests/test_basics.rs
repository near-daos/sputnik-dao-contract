use near_api::types::{AccessKey, AccessKeyPermission};
use near_api::{AccountId, NearToken};
use near_sandbox::config::{DEFAULT_GENESIS_ACCOUNT, DEFAULT_GENESIS_ACCOUNT_PRIVATE_KEY};
use near_sdk::serde_json::{json, Value};
use near_sdk::{
    base64::{engine::general_purpose, Engine as _},
    AccountIdRef,
};
use std::fs;

#[tokio::test]
async fn test_factory() -> Result<(), Box<dyn std::error::Error>> {
    const SPUTNIKDAO_FACTORY_CONTRACT_ACCOUNT: &AccountIdRef =
        AccountIdRef::new_or_panic("sputnik-dao.near");
    let sputnikdao_factory_contract =
        near_api::Contract(SPUTNIKDAO_FACTORY_CONTRACT_ACCOUNT.to_owned());

    let sandbox = near_sandbox::Sandbox::start_sandbox().await?;
    let sandbox_network =
        near_api::NetworkConfig::from_rpc_url("sandbox", sandbox.rpc_addr.parse()?);
    let signer = near_api::Signer::new(near_api::Signer::from_secret_key(
        DEFAULT_GENESIS_ACCOUNT_PRIVATE_KEY.parse()?,
    ))?;

    let mut account = near_api::Account(sputnikdao_factory_contract.0.clone())
        .view()
        .fetch_from_mainnet()
        .await?
        .data;
    account.amount = NearToken::from_near(50);
    sandbox
        .patch_state(sputnikdao_factory_contract.0.clone())
        .access_key(
            signer.get_public_key().await?.to_string(),
            AccessKey {
                nonce: 0.into(),
                permission: AccessKeyPermission::FullAccess,
            },
        )
        .account(account)
        .send()
        .await
        .unwrap();

    let wasm = fs::read("./res/sputnikdao_factory2.wasm").expect("Unable to read contract wasm");
    near_api::Contract::deploy(sputnikdao_factory_contract.0.clone())
        .use_code(wasm.to_vec())
        .with_init_call("new", ())
        .unwrap()
        .max_gas()
        .with_signer(signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

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

    let account_id: AccountId = format!("some_account.{}", DEFAULT_GENESIS_ACCOUNT)
        .parse()
        .unwrap();
    near_api::Account::create_account(account_id.clone())
        .fund_myself(DEFAULT_GENESIS_ACCOUNT.to_owned(), NearToken::from_near(20))
        .public_key(signer.clone().get_public_key().await?)
        .unwrap()
        .with_signer(signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    sputnikdao_factory_contract
        .call_function(
            "create",
            json!({
                "name": dao_name,
                "args":  general_purpose::STANDARD.encode(create_dao_args.to_string())
            }),
        )
        .unwrap()
        .transaction()
        .max_gas()
        .deposit(NearToken::from_near(6))
        .with_signer(account_id.clone(), signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    let dao_account_id: AccountId = format!("{}.{}", dao_name, SPUTNIKDAO_FACTORY_CONTRACT_ACCOUNT)
        .parse()
        .unwrap();

    let get_config_result: Value = near_api::Contract(dao_account_id)
        .call_function("get_config", ())
        .unwrap()
        .read_only()
        .fetch_from(&sandbox_network)
        .await
        .unwrap()
        .data;

    assert_eq!(create_dao_args["config"], get_config_result);

    Ok(())
}
