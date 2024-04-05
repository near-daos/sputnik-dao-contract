#![allow(dead_code)]
use std::convert::TryFrom;

pub use near_sdk::json_types::{Base64VecU8, ValidAccountId, WrappedDuration, U64};
use near_sdk::{AccountId, Balance};
use near_sdk_sim::transaction::ExecutionStatus;
use near_sdk_sim::{
    call, deploy, init_simulator, to_yocto, ContractAccount, ExecutionResult, UserAccount,
};

use near_sdk::json_types::U128;
use sputnik_staking::ContractContract as StakingContract;
use sputnikdao2::{
    Action, Config, ContractContract as DAOContract, ProposalInput, ProposalKind, VersionedPolicy,
};
use test_token::ContractContract as TestTokenContract;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    DAO_WASM_BYTES => "res/sputnikdao2.wasm",
    TEST_TOKEN_WASM_BYTES => "../test-token/res/test_token.wasm",
    STAKING_WASM_BYTES => "../sputnik-staking/res/sputnik_staking.wasm",
}

type Contract = ContractAccount<DAOContract>;

pub fn base_token() -> String {
    "".to_string()
}

pub fn should_fail(r: ExecutionResult) {
    match r.status() {
        ExecutionStatus::Failure(_) => {}
        _ => panic!("Should fail"),
    }
}

pub fn setup_dao() -> (UserAccount, Contract) {
    let root = init_simulator(None);
    let config = Config {
        name: "test".to_string(),
        purpose: "to test".to_string(),
        metadata: Base64VecU8(vec![]),
    };
    let dao = deploy!(
        contract: DAOContract,
        contract_id: "dao".to_string(),
        bytes: &DAO_WASM_BYTES,
        signer_account: root,
        deposit: to_yocto("200"),
        init_method: new(config, VersionedPolicy::Default(vec![root.account_id.clone()]))
    );
    (root, dao)
}

pub fn setup_test_token(root: &UserAccount) -> ContractAccount<TestTokenContract> {
    deploy!(
        contract: TestTokenContract,
        contract_id: "test_token".to_string(),
        bytes: &TEST_TOKEN_WASM_BYTES,
        signer_account: root,
        deposit: to_yocto("200"),
        init_method: new()
    )
}

pub fn setup_staking(root: &UserAccount) -> ContractAccount<StakingContract> {
    deploy!(
        contract: StakingContract,
        contract_id: "staking".to_string(),
        bytes: &STAKING_WASM_BYTES,
        signer_account: root,
        deposit: to_yocto("100"),
        init_method: new(to_va("dao".to_string()), to_va("test_token".to_string()), U64(100_000_000_000))
    )
}

pub fn add_proposal(
    root: &UserAccount,
    dao: &Contract,
    proposal: ProposalInput,
) -> ExecutionResult {
    call!(root, dao.add_proposal(proposal), deposit = to_yocto("1"))
}

pub fn add_member_proposal(
    root: &UserAccount,
    dao: &Contract,
    member_id: AccountId,
) -> ExecutionResult {
    add_proposal(
        root,
        dao,
        ProposalInput {
            description: "test".to_string(),
            kind: ProposalKind::AddMemberToRole {
                member_id: to_va(member_id),
                role: "council".to_string(),
            },
        },
    )
}

pub fn add_transfer_proposal(
    root: &UserAccount,
    dao: &Contract,
    token_id: AccountId,
    receiver_id: AccountId,
    amount: Balance,
    msg: Option<String>,
) -> ExecutionResult {
    add_proposal(
        root,
        dao,
        ProposalInput {
            description: "test".to_string(),
            kind: ProposalKind::Transfer {
                token_id,
                receiver_id: to_va(receiver_id),
                amount: U128(amount),
                msg,
            },
        },
    )
}

pub fn vote(users: Vec<&UserAccount>, dao: &Contract, proposal_id: u64) {
    for user in users.into_iter() {
        call!(
            user,
            dao.act_proposal(proposal_id, Action::VoteApprove, None)
        )
        .assert_success();
    }
}

pub fn to_va(a: AccountId) -> ValidAccountId {
    ValidAccountId::try_from(a).unwrap()
}
