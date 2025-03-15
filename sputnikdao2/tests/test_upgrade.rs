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

    let proposal_kind = ProposalKind::UpgradeSelf { hash };
    let proposal_id = root
        .call(&dao_account_id, "add_proposal")
        .args_json(json!({ "proposal": ProposalInput {
            description: "proposal to test".to_string(),
            kind: proposal_kind.clone()
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
        .args_json(json!({
            "id": 0,
            "action": Action::VoteApprove,
            "proposal": proposal_kind
        }))
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

/// Test that Sputnik can upgrade another contract.
#[tokio::test]
async fn test_upgrade_other() -> Result<(), Box<dyn std::error::Error>> {
    let (dao, worker, root) = setup_dao().await?;

    let ref_account = root
        .create_subaccount("ref-finance")
        .initial_balance(NearToken::from_near(2000))
        .transact()
        .await?
        .result;
    let ref_contract = ref_account.deploy(&OTHER_WASM_BYTES).await?.result;

    let ref_contract_new_result = ref_contract
        .call("new")
        .args_json(json!({
            "owner_id": dao.id(),
            "exchange_fee": 1,
            "referral_fee": 1,
        }))
        .transact()
        .await?;

    assert!(
        ref_contract_new_result.is_success(),
        "{:?}",
        ref_contract_new_result.failures()
    );

    let hash = root
        .call(dao.id(), "store_blob")
        .args(OTHER_WASM_BYTES.to_vec())
        .deposit(NearToken::from_near(200))
        .max_gas()
        .transact()
        .await?
        .json::<Base58CryptoHash>()
        .unwrap();
    assert!(add_proposal(
        &dao,
        ProposalInput {
            description: "test".to_string(),
            kind: ProposalKind::UpgradeRemote {
                receiver_id: near_sdk::AccountId::new_unchecked(ref_account.id().to_string()),
                method_name: "upgrade".to_string(),
                hash,
            },
        },
    )
    .await
    .is_success());

    let act_proposal_result = root
        .call(dao.id(), "act_proposal")
        .args_json(json!({
            "id": 0,
            "action": Action::VoteApprove,
            "proposal": get_proposal_kind(&dao, 0).await
        }))
        .max_gas()
        .transact()
        .await?;

    assert_eq!(
        0,
        act_proposal_result.failures().len(),
        "{:?}",
        act_proposal_result.failures()
    );
    assert!(act_proposal_result.is_success());

    Ok(())
}
