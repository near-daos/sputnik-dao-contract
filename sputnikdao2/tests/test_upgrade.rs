use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::Base58CryptoHash;
use near_sdk::serde_json::json;
use near_sdk::AccountId;

use near_sdk_sim::{call, init_simulator, to_yocto, DEFAULT_GAS};
use sputnikdao2::{Action, Config, ProposalInput, ProposalKind, VersionedPolicy};

mod utils;
use crate::utils::*;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    DAO_WASM_BYTES => "res/sputnikdao2.wasm",
    OTHER_WASM_BYTES => "res/ref_exchange_release.wasm"
}

#[test]
fn test_upgrade_using_factory() {
    let root = init_simulator(None);
    let factory = setup_factory(&root);
    factory
        .user_account
        .call(
            factory.user_account.account_id.clone(),
            "new",
            &[],
            near_sdk_sim::DEFAULT_GAS,
            0,
        )
        .assert_success();

    let config = Config {
        name: "testdao".to_string(),
        purpose: "to test".to_string(),
        metadata: Base64VecU8(vec![]),
    };
    let policy = VersionedPolicy::Default(vec![root.account_id()]);
    let params = json!({ "config": config, "policy": policy })
        .to_string()
        .into_bytes();

    call!(
        root,
        factory.create(
            AccountId::new_unchecked("testdao".to_string()),
            Base64VecU8(params)
        ),
        deposit = to_yocto("10")
    )
    .assert_success();

    let dao_account_id = AccountId::new_unchecked("testdao.factory".to_string());
    let dao_list = factory
        .user_account
        .view(factory.user_account.account_id.clone(), "get_dao_list", &[])
        .unwrap_json::<Vec<AccountId>>();
    assert_eq!(dao_list, vec![dao_account_id.clone()]);

    let hash = factory
        .user_account
        .view(
            factory.user_account.account_id.clone(),
            "get_default_code_hash",
            &[],
        )
        .unwrap_json::<Base58CryptoHash>();

    let proposal_id = root
        .call(
            dao_account_id.clone(),
            "add_proposal",
            &json!({ "proposal": ProposalInput {
                description: "proposal to test".to_string(),
                kind: ProposalKind::UpgradeSelf { hash }
            }})
            .to_string()
            .into_bytes(),
            near_sdk_sim::DEFAULT_GAS,
            to_yocto("1"),
        )
        .unwrap_json::<u64>();
    assert_eq!(0, proposal_id);

    root.call(
        dao_account_id.clone(),
        "act_proposal",
        &json!({ "id": 0, "action": Action::VoteApprove})
            .to_string()
            .into_bytes(),
        near_sdk_sim::DEFAULT_GAS,
        0,
    )
    .assert_success();
}

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
