use near_sdk::json_types::Base64VecU8;
use near_sdk_sim::{call, deploy, init_simulator, to_yocto, view};
use sputnikdao2::{
    Action, Config, ContractContract as Contract, ProposalInput, ProposalKind, VersionedPolicy,
};

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    DAO_WASM_BYTES => "res/sputnikdao2.wasm"
}

#[test]
fn test_upgrade() {
    let root = init_simulator(None);
    let config = Config {
        name: "test".to_string(),
        symbol: "TEST".to_string(),
        icon: None,
        reference: None,
        reference_hash: None,
        decimals: 24,
        purpose: "to test".to_string(),
        metadata: Base64VecU8(vec![]),
    };
    let dao = deploy!(
        contract: Contract,
        contract_id: "dao".to_string(),
        bytes: &DAO_WASM_BYTES,
        signer_account: root,
        deposit: to_yocto("200"),
        init_method: new(config, VersionedPolicy::Default(vec![root.account_id.clone()]))
    );
    let hash = root
        .call(
            dao.user_account.account_id.clone(),
            "store_blob",
            &DAO_WASM_BYTES,
            near_sdk_sim::DEFAULT_GAS,
            to_yocto("200"),
        )
        .unwrap_json::<Base64VecU8>();
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
    call!(root, dao.act_proposal(0, Action::VoteApprove)).assert_success();
    assert_eq!(view!(dao.version()).unwrap_json::<String>(), "2.0.0");
}
