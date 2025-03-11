use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::Base58CryptoHash;
use near_sdk::serde_json::json;

use near_workspaces::types::NearToken;
use near_workspaces::AccountId;
use sputnikdao2::{Action, Config, ProposalInput, ProposalKind, VersionedPolicy};

mod utils;
use crate::utils::*;

pub static DAO_WASM_BYTES: &[u8] = include_bytes!("../res/sputnikdao2.wasm");
pub static OTHER_WASM_BYTES: &[u8] = include_bytes!("../res/ref_exchange_release.wasm");

#[tokio::test]
async fn test_upgrade_using_factory() -> Result<(), Box<dyn std::error::Error>> {
    let (factory, worker) = setup_factory().await?;
    let root = worker.root_account().unwrap();

    let config = Config {
        name: "testdao".to_string(),
        purpose: "to test".to_string(),
        metadata: Base64VecU8(vec![]),
    };
    let policy = VersionedPolicy::Default(vec![near_sdk::AccountId::new_unchecked(
        root.id().to_string(),
    )]);
    let params = json!({ "config": config, "policy": policy })
        .to_string()
        .into_bytes();

    let create_result = root
        .call(factory.id(), "create")
        .args_json(json!({
            "name": "testdao",
            "args": Base64VecU8(params)
        }))
        .deposit(NearToken::from_near(10))
        .max_gas()
        .transact()
        .await?;
    assert!(create_result.is_success(), "{:?}", create_result.failures());

    let dao_account_id: AccountId = format!("testdao.{}", factory.id().to_string())
        .parse()
        .unwrap();
    let dao_list = factory
        .view("get_dao_list")
        .await?
        .json::<Vec<near_sdk::AccountId>>()
        .unwrap();
    assert_eq!(
        dao_list,
        vec![near_sdk::AccountId::new_unchecked(
            dao_account_id.to_string()
        )]
    );

    let hash = factory
        .view("get_default_code_hash")
        .await?
        .json::<Base58CryptoHash>()
        .unwrap();

    let proposal_id = root
        .call(&dao_account_id, "add_proposal")
        .args_json(json!({ "proposal": ProposalInput {
            description: "proposal to test".to_string(),
            kind: ProposalKind::UpgradeSelf { hash }
        }}))
        .deposit(NearToken::from_near(1))
        .transact()
        .await?
        .unwrap()
        .json::<u64>()
        .unwrap();

    assert_eq!(0, proposal_id);

    let act_proposal_result = root
        .call(&dao_account_id, "act_proposal")
        .args_json(json!({"id": 0, "action": Action::VoteApprove}))
        .max_gas()
        .transact()
        .await?;
    assert!(
        act_proposal_result.is_success(),
        "{:?}",
        act_proposal_result.failures()
    );
    assert_eq!(
        0,
        act_proposal_result.failures().len(),
        "{:?}",
        act_proposal_result.failures()
    );

    Ok(())
}

/*
#[derive(BorshSerialize, BorshDeserialize)]
struct NewArgs {
    owner_id: AccountId,
    exchange_fee: u32,
    referral_fee: u32,
}



/// Test that Sputnik can upgrade another contract.
#[test]
fn test_upgrade_other() {
    let (root, dao) = setup_dao();
    let ref_account_id: AccountId = "ref-finance".parse().unwrap();
    let _ = root.deploy_and_init(
        &OTHER_WASM_BYTES,
        ref_account_id.clone(),
        "new",
        &json!({
            "owner_id": dao.account_id(),
            "exchange_fee": 1,
            "referral_fee": 1,
        })
        .to_string()
        .into_bytes(),
        to_yocto("1000"),
        DEFAULT_GAS,
    );
    let hash = root
        .call(
            dao.user_account.account_id.clone(),
            "store_blob",
            &OTHER_WASM_BYTES,
            near_sdk_sim::DEFAULT_GAS,
            to_yocto("200"),
        )
        .unwrap_json::<Base58CryptoHash>();
    add_proposal(
        &root,
        &dao,
        ProposalInput {
            description: "test".to_string(),
            kind: ProposalKind::UpgradeRemote {
                receiver_id: ref_account_id.clone(),
                method_name: "upgrade".to_string(),
                hash,
            },
        },
    )
    .assert_success();
    call!(root, dao.act_proposal(0, Action::VoteApprove, None)).assert_success();
}

*/
