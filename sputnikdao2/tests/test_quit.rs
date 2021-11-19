#![allow(clippy::ref_in_deref)]
#![allow(clippy::identity_op)]

use crate::utils::{
    add_member_to_role_proposal, add_proposal, setup_dao, should_fail_with, vote, Contract,
};
use near_sdk::AccountId;
use near_sdk_sim::{call, to_yocto, view};
use near_sdk_sim::{ExecutionResult, UserAccount};
use sputnikdao2::{
    Action, Policy, Proposal, ProposalInput, ProposalKind, ProposalPermission, ProposalStatus,
    RoleKind, RolePermission, VersionedPolicy,
};
use std::collections::HashMap;
use std::collections::HashSet;

mod utils;

const KILO: u128 = 1000;
const MEGA: u128 = KILO * KILO;
const YOTTA: u128 = MEGA * MEGA * MEGA * MEGA;

fn user(id: u32) -> AccountId {
    format!("user{}", id).parse().unwrap()
}

fn new_role(name: String, permissions: HashSet<ProposalPermission>) -> RolePermission {
    RolePermission {
        name,
        kind: RoleKind::Group(HashSet::new()),
        permissions,
        vote_policy: HashMap::new(),
    }
}

/// Updates the policy, pushing the roles into it.
fn policy_extend_roles(root: &UserAccount, dao: &Contract, roles: Vec<RolePermission>) {
    {
        let mut policy = view!(dao.get_policy()).unwrap_json::<Policy>();
        policy.roles.extend(roles);
        add_proposal(
            root,
            dao,
            ProposalInput {
                description: "new_policy".to_string(),
                kind: ProposalKind::ChangePolicy {
                    policy: VersionedPolicy::Current(policy.clone()),
                },
            },
        )
        .assert_success();
        let change_policy = view!(dao.get_last_proposal_id()).unwrap_json::<u64>();
        assert_eq!(change_policy, 1);
        call!(
            root,
            dao.act_proposal(change_policy - 1, Action::VoteApprove, None)
        )
        .assert_success();
    };
}

fn add_user_to_roles(
    root: &UserAccount,
    dao: &Contract,
    user: &UserAccount,
    role_names: Vec<&str>,
) {
    for role_name in role_names {
        add_member_to_role_proposal(root, dao, user.account_id.clone(), role_name.to_string())
            .assert_success();

        // approval
        let proposal = view!(dao.get_last_proposal_id()).unwrap_json::<u64>();
        call!(
            root,
            dao.act_proposal(proposal - 1, Action::VoteApprove, None)
        )
        .assert_success();
    }
}

/// Given a RolePermission, get it's members in a sorted `Vec`.
fn role_members(role_permission: &sputnikdao2::RolePermission) -> Vec<AccountId> {
    if let RoleKind::Group(ref members) = role_permission.kind {
        let mut members = members.iter().cloned().collect::<Vec<_>>();
        members.sort();
        members
    } else {
        vec![]
    }
}

type RoleNamesAndMembers = Vec<(String, Vec<AccountId>)>;

/// Get dao role names and their members
fn dao_roles(dao: &Contract) -> RoleNamesAndMembers {
    view!(dao.get_policy())
        .unwrap_json::<Policy>()
        .roles
        .into_iter()
        .map(|role_permission| (role_permission.name.clone(), role_members(&role_permission)))
        .collect()
}

type RoleNamesAndMembersRef<'a> = Vec<(&'a str, Vec<&'a AccountId>)>;
/// Makes references into a `RoleNamesAndMembers`
/// so they are easier to compare against.
#[allow(clippy::ptr_arg)]
fn dao_roles_ref(dao_roles: &RoleNamesAndMembers) -> RoleNamesAndMembersRef {
    dao_roles
        .iter()
        .map(|(name, members)| (name.as_str(), members.iter().collect()))
        .collect::<Vec<(&str, Vec<&AccountId>)>>()
}

/// Quit from the dao.
fn quit(
    dao: &Contract,
    user: &UserAccount,
    user_check: &UserAccount,
    dao_name_check: String,
) -> Result<bool, String> {
    use near_sdk_sim::transaction::ExecutionStatus;
    use near_sdk_sim::ExecutionResult;
    let res: ExecutionResult = call!(
        user,
        dao.quit_from_all_roles(user_check.account_id.clone(), dao_name_check),
        deposit = to_yocto("0")
    );
    match res.status() {
        ExecutionStatus::SuccessValue(_bytes) => Ok(res.unwrap_json::<bool>()),
        ExecutionStatus::Failure(err) => Err(err.to_string()),
        _ => panic!("unexpected status"),
    }
}

/// Adds some dummy proposal, for the votes to be tested on.  
/// (transfers of 1 yocto-near to `receiver`).
fn add_transfer_proposal(root: &UserAccount, dao: &Contract, receiver: &UserAccount) -> u64 {
    let proposal_input = ProposalInput {
        description: "new policy".to_string(),
        kind: ProposalKind::Transfer {
            token_id: None,
            receiver_id: receiver.account_id(),
            amount: 1u128.into(),
            msg: None,
        },
    };
    call!(root, dao.add_proposal(proposal_input), deposit = 1 * YOTTA).unwrap_json::<u64>()
}

/// Issue #41 "Quitting the DAO" tests
#[test]
fn test_quitting_the_dao() {
    let (root, dao) = setup_dao();
    let user2 = root.create_user(user(2), to_yocto("1000"));
    let user3 = root.create_user(user(3), to_yocto("1000"));
    let user4 = root.create_user(user(4), to_yocto("1000"));

    let role_none = new_role("has_nobody".to_string(), HashSet::new());
    let role_2 = new_role("has_2".to_string(), HashSet::new());
    let role_3 = new_role("has_3".to_string(), HashSet::new());
    let role_23 = new_role("has_23".to_string(), HashSet::new());
    let role_234 = new_role("has_234".to_string(), HashSet::new());

    policy_extend_roles(
        &root,
        &dao,
        vec![role_none, role_2, role_3, role_23, role_234],
    );

    add_user_to_roles(&root, &dao, &user2, vec!["has_2", "has_23", "has_234"]);
    add_user_to_roles(&root, &dao, &user3, vec!["has_3", "has_23", "has_234"]);
    add_user_to_roles(&root, &dao, &user4, vec!["has_234"]);

    // initial check,
    // when nobody has quit yet
    let roles = dao_roles(&dao);
    assert_eq!(
        dao_roles_ref(&roles),
        vec![
            ("all", vec![]),
            ("council", vec![&root.account_id]),
            ("has_nobody", vec![]),
            ("has_2", vec![&user2.account_id,]),
            ("has_3", vec![&user3.account_id]),
            ("has_23", vec![&user2.account_id, &user3.account_id]),
            (
                "has_234",
                vec![&user2.account_id, &user3.account_id, &user4.account_id]
            )
        ]
    );

    let config = view!(dao.get_config()).unwrap_json::<sputnikdao2::Config>();
    let dao_name = &config.name;

    // ok: user2 quits
    let res = quit(&dao, &user2, &user2, dao_name.clone()).unwrap();
    assert!(res);
    let roles = dao_roles(&dao);
    assert_eq!(
        dao_roles_ref(&roles),
        vec![
            ("all", vec![]),
            ("council", vec![&root.account_id]),
            ("has_nobody", vec![]),
            ("has_2", vec![]),
            ("has_3", vec![&user3.account_id]),
            ("has_23", vec![&user3.account_id]),
            ("has_234", vec![&user3.account_id, &user4.account_id])
        ]
    );

    // ok: user2 quits again
    // (makes no change)
    let res = quit(&dao, &user2, &user2, dao_name.clone()).unwrap();
    assert!(!res);
    let roles = dao_roles(&dao);
    assert_eq!(
        dao_roles_ref(&roles),
        vec![
            ("all", vec![]),
            ("council", vec![&root.account_id]),
            ("has_nobody", vec![]),
            ("has_2", vec![]),
            ("has_3", vec![&user3.account_id]),
            ("has_23", vec![&user3.account_id]),
            ("has_234", vec![&user3.account_id, &user4.account_id])
        ]
    );

    // fail: user3 quits passing the wrong user name
    let res = quit(&dao, &user3, &user2, dao_name.clone()).unwrap_err();
    assert_eq!(
        res,
        "Action #0: Smart contract panicked: ERR_QUIT_WRONG_ACC"
    );
    let roles = dao_roles(&dao);
    assert_eq!(
        dao_roles_ref(&roles),
        vec![
            ("all", vec![]),
            ("council", vec![&root.account_id]),
            ("has_nobody", vec![]),
            ("has_2", vec![]),
            ("has_3", vec![&user3.account_id]),
            ("has_23", vec![&user3.account_id]),
            ("has_234", vec![&user3.account_id, &user4.account_id])
        ]
    );

    // fail: user3 quits passing the wrong dao name
    let wrong_dao_name = format!("wrong_{}", &dao_name);
    let res = quit(&dao, &user3, &user3, wrong_dao_name).unwrap_err();
    assert_eq!(
        res,
        "Action #0: Smart contract panicked: ERR_QUIT_WRONG_DAO"
    );
    let roles = dao_roles(&dao);
    assert_eq!(
        dao_roles_ref(&roles),
        vec![
            ("all", vec![]),
            ("council", vec![&root.account_id]),
            ("has_nobody", vec![]),
            ("has_2", vec![]),
            ("has_3", vec![&user3.account_id]),
            ("has_23", vec![&user3.account_id]),
            ("has_234", vec![&user3.account_id, &user4.account_id])
        ]
    );

    // ok: user3 quits
    let res = quit(&dao, &user3, &user3, dao_name.clone()).unwrap();
    assert!(res);
    let roles = dao_roles(&dao);
    assert_eq!(
        dao_roles_ref(&roles),
        vec![
            ("all", vec![]),
            ("council", vec![&root.account_id]),
            ("has_nobody", vec![]),
            ("has_2", vec![]),
            ("has_3", vec![]),
            ("has_23", vec![]),
            ("has_234", vec![&user4.account_id])
        ]
    );
}

/// Tests a role with Ratio = 1/2 with two members,
/// when one member votes and then the other one quits.  
/// There should be a way for the users to "finalize"
/// the decision on the proposal, since it would now only
/// require that single vote.
///
/// https://github.com/near-daos/sputnik-dao-contract/issues/41#issuecomment-970170648
#[test]
fn test_quit_removes_votes1() {
    let (root, dao) = setup_dao();
    let user2 = root.create_user(user(2), to_yocto("1000"));
    let user3 = root.create_user(user(3), to_yocto("1000"));
    let user4 = root.create_user(user(4), to_yocto("1000"));

    // users (2, 3) will share a role,
    // and only user2 will vote in approval, then user3 quits.
    // then assert that the proposals can get approved from only 1 vote.

    let dao_name = {
        let config = view!(dao.get_config()).unwrap_json::<sputnikdao2::Config>();
        config.name
    };

    {
        let permissions = vec!["*:*".to_string()].into_iter().collect();
        let role_23 = new_role("has_23".to_string(), permissions);
        policy_extend_roles(&root, &dao, vec![role_23]);
    }

    add_user_to_roles(&root, &dao, &user2, vec!["has_23"]);
    add_user_to_roles(&root, &dao, &user3, vec!["has_23"]);

    // adds two transfer proposals
    let t1 = add_transfer_proposal(&root, &dao, &user4);
    let t2 = add_transfer_proposal(&root, &dao, &user4);

    // user2 votes in approval of both
    vote(vec![&user2], &dao, t1);
    vote(vec![&user2], &dao, t2);

    // user3 quits role
    let res = quit(&dao, &user3, &user3, dao_name).unwrap();
    assert!(res);

    // user2 finalizes t1
    let user4amount = user4.account().unwrap().amount;
    call!(user2, dao.act_proposal(t1, Action::Finalize, None)).assert_success();
    panic!("checkpoint A");
    assert_eq!(
        view!(dao.get_proposal(t1)).unwrap_json::<Proposal>().status,
        ProposalStatus::Approved
    );
    // confirm user4 received the transfer
    assert_eq!(
        user4amount
       // the bounty
       + 1,
        user4.account().unwrap().amount
    );

    // user3 finalizes t2
    let user4amount = user4.account().unwrap().amount;
    call!(user3, dao.act_proposal(t2, Action::Finalize, None)).assert_success();
    assert_eq!(
        view!(dao.get_proposal(t2)).unwrap_json::<Proposal>().status,
        ProposalStatus::Approved
    );
    // confirm user4 received the transfer
    assert_eq!(
        user4amount
       // the bounty
       + 1,
        user4.account().unwrap().amount
    );
}

/// Tests a role with Ratio = 1/2 with two members,
/// when one member votes and then quits.  
/// That single vote should not cause (nor allow) a state change
/// in the proposal. That vote should be removed instead.
///
/// https://github.com/near-daos/sputnik-dao-contract/issues/41#issuecomment-971474598
#[test]
fn test_quit_removes_votes2() {
    let (root, dao) = setup_dao();
    let user2 = root.create_user(user(2), to_yocto("1000"));
    let user3 = root.create_user(user(3), to_yocto("1000"));
    let user4 = root.create_user(user(4), to_yocto("1000"));

    // users (2, 3) will share a role,
    // and only user2 will vote in approval and then quit.
    // then assert that the proposals cannot get approved from only 1 vote.

    let dao_name = {
        let config = view!(dao.get_config()).unwrap_json::<sputnikdao2::Config>();
        config.name
    };

    {
        let permissions = vec!["*:*".to_string()].into_iter().collect();
        let role_23 = new_role("has_23".to_string(), permissions);
        policy_extend_roles(&root, &dao, vec![role_23]);
    }

    add_user_to_roles(&root, &dao, &user2, vec!["has_23"]);
    add_user_to_roles(&root, &dao, &user3, vec!["has_23"]);

    // adds two transfer proposals
    let t1 = add_transfer_proposal(&root, &dao, &user4);
    let t2 = add_transfer_proposal(&root, &dao, &user4);

    // user2 votes in approval of both
    vote(vec![&user2], &dao, t1);
    vote(vec![&user2], &dao, t2);

    // user2 quits role
    let res = quit(&dao, &user2, &user2, dao_name).unwrap();
    assert!(res);

    // user2 tries to finalize t1
    let res = call!(user2, dao.act_proposal(t1, Action::Finalize, None));
    should_fail_with(res, 0, "ERR_FINALIZE");
    // confirm t1 did not get approved
    assert_eq!(
        view!(dao.get_proposal(t1)).unwrap_json::<Proposal>().status,
        ProposalStatus::InProgress
    );

    // user3 tries to finalize t2
    let res = call!(user3, dao.act_proposal(t2, Action::Finalize, None));
    should_fail_with(res, 0, "ERR_FINALIZE");
    // confirm t2 did not get approved
    assert_eq!(
        view!(dao.get_proposal(t2)).unwrap_json::<Proposal>().status,
        ProposalStatus::InProgress
    );
}
