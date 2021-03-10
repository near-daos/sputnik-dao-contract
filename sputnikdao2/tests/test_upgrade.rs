use near_sdk::borsh::BorshSerialize;
use near_sdk::json_types::U128;
use near_sdk::PendingContractTx;
use near_sdk_sim::{call, deploy, init_simulator, to_yocto, view};
use sputnikdao2::{Action, Config, ContractContract as Contract};

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    DAO_WASM_BYTES => "res/sputnikdao2.wasm"
}

#[test]
fn test_upgrade() {
    let root = init_simulator(None);
    let config = Config {
        name: "test".to_string(),
        purpose: "to test".to_string(),
        bond: U128(to_yocto("1")),
        symbol: "TEST".to_string(),
        decimals: 24,
    };
    let dao = deploy!(
        contract: Contract,
        contract_id: "dao".to_string(),
        bytes: &DAO_WASM_BYTES,
        signer_account: root,
        deposit: to_yocto("100"),
        init_method: new(config, None)
    );
    root.call(
        PendingContractTx {
            receiver_id: dao.user_account.account_id.clone(),
            method: "stage_code".to_string(),
            args: DAO_WASM_BYTES.try_to_vec().unwrap(),
            is_view: false,
        },
        to_yocto("100"),
        near_sdk_sim::DEFAULT_GAS,
    )
    .assert_success();
    assert_eq!(view!(dao.get_last_proposal_id()).unwrap_json::<u64>(), 1);
    call!(root, dao.act_proposal(0, Action::VoteApprove)).assert_success();
}
