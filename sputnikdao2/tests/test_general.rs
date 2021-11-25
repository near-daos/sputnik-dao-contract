use std::collections::HashMap;

use near_sdk::json_types::U128;
use near_sdk::AccountId;
use near_sdk_sim::{call, to_yocto, view};

use crate::utils::*;
use sputnik_staking::User;
use sputnikdao2::{
    Action, Policy, Proposal, ProposalInput, ProposalKind, ProposalStatus, RoleKind,
    RolePermission, VersionedPolicy, VotePolicy,
};

mod utils;

fn user(id: u32) -> AccountId {
    format!("user{}", id).parse().unwrap()
}

#[test]
fn test_multi_council() {
    let (root, dao) = setup_dao();
    let user1 = root.create_user(user(1), to_yocto("1000"));
    let user2 = root.create_user(user(2), to_yocto("1000"));
    let user3 = root.create_user(user(3), to_yocto("1000"));
    let new_policy = Policy {
        roles: vec![
            RolePermission {
                name: "all".to_string(),
                kind: RoleKind::Everyone,
                permissions: vec!["*:AddProposal".to_string()].into_iter().collect(),
                vote_policy: HashMap::default(),
            },
            RolePermission {
                name: "council".to_string(),
                kind: RoleKind::Group(vec![user(1), user(2)].into_iter().collect()),
                permissions: vec!["*:*".to_string()].into_iter().collect(),
                vote_policy: HashMap::default(),
            },
            RolePermission {
                name: "community".to_string(),
                kind: RoleKind::Group(vec![user(1), user(3), user(4)].into_iter().collect()),
                permissions: vec!["*:*".to_string()].into_iter().collect(),
                vote_policy: HashMap::default(),
            },
        ],
        default_vote_policy: VotePolicy::default(),
        proposal_bond: U128(10u128.pow(24)),
        proposal_period: U64::from(1_000_000_000 * 60 * 60 * 24 * 7),
        bounty_bond: U128(10u128.pow(24)),
        bounty_forgiveness_period: U64::from(1_000_000_000 * 60 * 60 * 24),
    };
    add_proposal(
        &root,
        &dao,
        ProposalInput {
            description: "new policy".to_string(),
            kind: ProposalKind::ChangePolicy {
                policy: VersionedPolicy::Current(new_policy.clone()),
            },
        },
    )
    .assert_success();
    vote(vec![&root], &dao, 0);
    assert_eq!(view!(dao.get_policy()).unwrap_json::<Policy>(), new_policy);
    add_transfer_proposal(&root, &dao, base_token(), user(1), 1_000_000, None).assert_success();
    vote(vec![&user2], &dao, 1);
    vote(vec![&user3], &dao, 1);
    let proposal = view!(dao.get_proposal(1)).unwrap_json::<Proposal>();
    // Votes from members in different councils.
    assert_eq!(proposal.status, ProposalStatus::InProgress);
    // Finish with vote that is in both councils, which approves the proposal.
    vote(vec![&user1], &dao, 1);
    let proposal = view!(dao.get_proposal(1)).unwrap_json::<Proposal>();
    assert_eq!(proposal.status, ProposalStatus::Approved);
}

#[test]
fn test_create_dao_and_use_token() {
    let (root, dao) = setup_dao();
    let user2 = root.create_user(user(2), to_yocto("1000"));
    let user3 = root.create_user(user(3), to_yocto("1000"));
    let test_token = setup_test_token(&root);
    let staking = setup_staking(&root);

    assert!(view!(dao.get_staking_contract())
        .unwrap_json::<Option<AccountId>>()
        .is_none());
    add_member_proposal(&root, &dao, user2.account_id.clone()).assert_success();
    assert_eq!(view!(dao.get_last_proposal_id()).unwrap_json::<u64>(), 1);
    // Voting by user who is not member should fail.
    should_fail(call!(user2, dao.act_proposal(0, Action::VoteApprove, None)));
    call!(root, dao.act_proposal(0, Action::VoteApprove, None)).assert_success();
    // voting second time should fail.
    should_fail(call!(root, dao.act_proposal(0, Action::VoteApprove, None)));
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
    add_proposal(
        &user2,
        &dao,
        ProposalInput {
            description: "test".to_string(),
            kind: ProposalKind::SetStakingContract {
                staking_id: "staking".parse().unwrap(),
            },
        },
    )
    .assert_success();
    vote(vec![&user3, &user2], &dao, 2);
    assert!(!view!(dao.get_staking_contract())
        .unwrap_json::<Option<AccountId>>()
        .is_none());
    assert_eq!(
        view!(dao.get_proposal(2)).unwrap_json::<Proposal>().status,
        ProposalStatus::Approved
    );

    staking
        .user_account
        .view_method_call(staking.contract.ft_total_supply());
    assert_eq!(
        view!(staking.ft_total_supply()).unwrap_json::<U128>().0,
        to_yocto("0")
    );
    call!(
        user2,
        test_token.mint(user2.account_id.clone(), U128(to_yocto("100")))
    )
    .assert_success();
    call!(
        user2,
        test_token.storage_deposit(Some(staking.account_id()), None),
        deposit = to_yocto("1")
    )
    .assert_success();
    call!(
        user2,
        staking.storage_deposit(None, None),
        deposit = to_yocto("1")
    );
    call!(
        user2,
        test_token.ft_transfer_call(
            staking.account_id(),
            U128(to_yocto("10")),
            None,
            "".to_string()
        ),
        deposit = 1
    )
    .assert_success();
    assert_eq!(
        view!(staking.ft_total_supply()).unwrap_json::<U128>().0,
        to_yocto("10")
    );
    let user2_id = user2.account_id.clone();
    assert_eq!(
        view!(staking.ft_balance_of(user2_id.clone()))
            .unwrap_json::<U128>()
            .0,
        to_yocto("10")
    );
    assert_eq!(
        view!(test_token.ft_balance_of(user2_id.clone()))
            .unwrap_json::<U128>()
            .0,
        to_yocto("90")
    );
    call!(user2, staking.withdraw(U128(to_yocto("5")))).assert_success();
    assert_eq!(
        view!(staking.ft_total_supply()).unwrap_json::<U128>().0,
        to_yocto("5")
    );
    assert_eq!(
        view!(test_token.ft_balance_of(user2_id.clone()))
            .unwrap_json::<U128>()
            .0,
        to_yocto("95")
    );
    call!(
        user2,
        staking.delegate(user2_id.clone(), U128(to_yocto("5")))
    )
    .assert_success();
    call!(
        user2,
        staking.undelegate(user2_id.clone(), U128(to_yocto("1")))
    )
    .assert_success();
    // should fail right after undelegation as need to wait for voting period before can delegate again.
    should_fail(call!(
        user2,
        staking.delegate(user2_id.clone(), U128(to_yocto("1")))
    ));
    let user = view!(staking.get_user(user2_id.clone())).unwrap_json::<User>();
    assert_eq!(
        user.delegated_amounts,
        vec![(user2_id.clone(), U128(to_yocto("4")))]
    );
    assert_eq!(
        view!(dao.delegation_total_supply()).unwrap_json::<U128>().0,
        to_yocto("4")
    );
    assert_eq!(
        view!(dao.delegation_balance_of(user2_id.clone()))
            .unwrap_json::<U128>()
            .0,
        to_yocto("4")
    );
}

/// Test various cases that must fail.
#[test]
fn test_failures() {
    let (root, dao) = setup_dao();
    should_fail(add_transfer_proposal(
        &root,
        &dao,
        base_token(),
        user(1),
        1_000_000,
        Some("some".to_string()),
    ));
}

/// Test payments that fail
#[test]
fn test_payment_failures() {
    let (root, dao) = setup_dao();
    let user1 = root.create_user(user(1), to_yocto("1000"));
    let whale = root.create_user(user(2), to_yocto("1000"));

    // Add user1
    add_member_proposal(&root, &dao, user1.account_id.clone()).assert_success();
    vote(vec![&root], &dao, 0);

    // Set up fungible tokens and give 5 to the dao
    let test_token = setup_test_token(&root);
    call!(
        dao.user_account,
        test_token.mint(dao.user_account.account_id.clone(), U128(5))
    )
    .assert_success();
    call!(
        user1,
        test_token.storage_deposit(Some(user1.account_id.clone()), Some(true)),
        deposit = to_yocto("125")
    )
    .assert_success();

    // Attempt to transfer more than it has
    add_transfer_proposal(
        &root,
        &dao,
        Some(test_token.account_id()),
        user(1),
        10,
        None,
    )
    .assert_success();

    // Vote in the transfer
    vote(vec![&root, &user1], &dao, 1);
    let mut proposal = view!(dao.get_proposal(1)).unwrap_json::<Proposal>();
    assert_eq!(proposal.status, ProposalStatus::Failed);

    // Set up benefactor whale who will donate the needed tokens
    call!(
        whale,
        test_token.mint(whale.account_id.clone(), U128(6_000_000_000))
    )
    .assert_success();
    call!(
        whale,
        test_token.ft_transfer(
            dao.account_id(),
            U128::from(1000),
            Some("Heard you're in a pinch, let me help.".to_string())
        ),
        deposit = 1
    )
    .assert_success();

    // Council member retries payment via an action
    call!(
        root,
        dao.act_proposal(
            1,
            Action::Finalize,
            Some("Sorry! We topped up our tokens. Thanks.".to_string())
        )
    )
    .assert_success();

    proposal = view!(dao.get_proposal(1)).unwrap_json::<Proposal>();
    assert_eq!(
        proposal.status,
        ProposalStatus::Approved,
        "Did not return to approved status."
    );
}
