use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::Base58CryptoHash;
use near_sdk::serde_json::json;
use near_sdk::AccountId;

use near_sdk_sim::{call, to_yocto, view, DEFAULT_GAS};
use sputnikdao2::{Action, ProposalInput, ProposalKind};

mod utils;
use crate::utils::*;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    DAO_WASM_BYTES => "res/sputnikdao2.wasm",
    OTHER_WASM_BYTES => "res/ref_exchange_release.wasm"
}

#[test]
fn test_upgrade() {
    let (root, dao) = setup_dao();
    let hash = root
        .call(
            dao.user_account.account_id.clone(),
            "store_blob",
            &DAO_WASM_BYTES,
            near_sdk_sim::DEFAULT_GAS,
            to_yocto("200"),
        )
        .unwrap_json::<Base58CryptoHash>();
    call!(
        root,
        dao.add_proposal(ProposalInput {
            description: "test".to_string(),
            kind: ProposalKind::UpgradeSelf { hash }
        }),
        deposit = to_yocto("1")
    )
    .assert_success();
    assert_eq!(view!(dao.get_last_proposal_id()).unwrap_json::<u64>(), 1);
    call!(root, dao.act_proposal(0, Action::VoteApprove, None)).assert_success();
    assert_eq!(view!(dao.version()).unwrap_json::<String>(), "2.0.0");
    call!(root, dao.remove_blob(hash)).assert_success();
    should_fail(call!(root, dao.remove_blob(hash)));
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
    let ref_account_id : AccountId = "ref-finance".parse().unwrap();
    let _ = root.deploy_and_init(
        &OTHER_WASM_BYTES,
        ref_account_id.clone(),
        "new",
        &json!({
            "owner_id": to_va(dao.account_id()),
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
                receiver_id: to_va(ref_account_id.clone()),
                method_name: "upgrade".to_string(),
                hash,
            },
        },
    )
    .assert_success();
    call!(root, dao.act_proposal(0, Action::VoteApprove, None)).assert_success();
}
