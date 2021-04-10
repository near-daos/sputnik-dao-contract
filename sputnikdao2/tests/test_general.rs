use near_sdk::json_types::{Base64VecU8, U128};
use near_sdk::{AccountId, Balance};
use near_sdk_sim::transaction::ExecutionStatus;
use near_sdk_sim::{
    call, deploy, init_simulator, to_yocto, view, ContractAccount, ExecutionResult, UserAccount,
};

use sputnikdao2::{
    Action, Config, ContractContract as DAOContract, Policy, Proposal, ProposalInput, ProposalKind,
    ProposalStatus, RoleKind,
};

type Contract = ContractAccount<DAOContract>;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    DAO_WASM_BYTES => "res/sputnikdao2.wasm"
}

fn should_fail(r: ExecutionResult) {
    match r.status() {
        ExecutionStatus::Failure(_) => {}
        _ => panic!("Should fail"),
    }
}

fn setup_dao() -> (UserAccount, Contract) {
    let root = init_simulator(None);
    let config = Config {
        name: "test".to_string(),
        symbol: "TEST".to_string(),
        icon: None,
        reference: None,
        reference_hash: None,
        decimals: 24,
        purpose: "to test".to_string(),
        bond: U128(to_yocto("1")),
        metadata: Base64VecU8(vec![]),
    };
    let dao = deploy!(
        contract: DAOContract,
        contract_id: "dao".to_string(),
        bytes: &DAO_WASM_BYTES,
        signer_account: root,
        deposit: to_yocto("200"),
        init_method: new(config, None)
    );
    (root, dao)
}

fn add_proposal(root: &UserAccount, dao: &Contract, proposal: ProposalInput) -> ExecutionResult {
    call!(root, dao.add_proposal(proposal), deposit = to_yocto("1"))
}

fn add_member_proposal(
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
                member_id,
                role: "council".to_string(),
            },
        },
    )
}

fn add_mint_proposal(root: &UserAccount, dao: &Contract, amount: Balance) -> ExecutionResult {
    add_proposal(
        root,
        &dao,
        ProposalInput {
            description: "test".to_string(),
            kind: ProposalKind::Mint {
                amount: U128(amount),
            },
        },
    )
}

fn add_burn_proposal(root: &UserAccount, dao: &Contract, amount: Balance) -> ExecutionResult {
    add_proposal(
        root,
        &dao,
        ProposalInput {
            description: "test".to_string(),
            kind: ProposalKind::Burn {
                amount: U128(amount),
            },
        },
    )
}

fn vote(users: Vec<&UserAccount>, dao: &Contract, proposal_id: u64) {
    for user in users.into_iter() {
        call!(user, dao.act_proposal(proposal_id, Action::VoteApprove)).assert_success();
    }
}

#[test]
fn test_create_dao_and_mint() {
    let (root, dao) = setup_dao();
    let user2 = root.create_user("user2".to_string(), to_yocto("1000"));
    let user3 = root.create_user("user3".to_string(), to_yocto("1000"));
    // Not a member can not submit adding members.
    should_fail(add_member_proposal(&user2, &dao, user2.account_id.clone()));
    add_member_proposal(&root, &dao, user2.account_id.clone()).assert_success();
    assert_eq!(view!(dao.get_last_proposal_id()).unwrap_json::<u64>(), 1);
    // Voting by user who is not member should fail.
    should_fail(call!(user2, dao.act_proposal(0, Action::VoteApprove)));
    call!(root, dao.act_proposal(0, Action::VoteApprove)).assert_success();
    // voting second time should fail.
    should_fail(call!(root, dao.act_proposal(0, Action::VoteApprove)));
    // Add 3rd member.
    add_member_proposal(&user2, &dao, user3.account_id.clone()).assert_success();
    vote(vec![&root, &user2], &dao, 1);
    let policy = view!(dao.get_policy()).unwrap_json::<Policy>();
    assert_eq!(policy.roles.len(), 2);
    assert_eq!(
        policy.roles[1].kind,
        RoleKind::Group(
            vec![
                root.account_id.clone(),
                user2.account_id.clone(),
                user3.account_id.clone()
            ]
            .into_iter()
            .collect()
        )
    );
    add_mint_proposal(&user2, &dao, to_yocto("1000")).assert_success();
    vote(vec![&user3, &user2], &dao, 2);
    assert_eq!(
        view!(dao.get_proposal(2)).unwrap_json::<Proposal>().status,
        ProposalStatus::Approved
    );
    assert_eq!(
        view!(dao.ft_total_supply()).unwrap_json::<U128>().0,
        to_yocto("1000")
    );
    add_burn_proposal(&user3, &dao, to_yocto("500"));
    vote(vec![&user3, &user2], &dao, 3);
    assert_eq!(
        view!(dao.ft_total_supply()).unwrap_json::<U128>().0,
        to_yocto("500")
    );
}
