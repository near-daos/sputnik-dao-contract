use near_sdk::base64;
use near_sdk::json_types::U128;
use near_sdk::serde_json::json;

use near_workspaces::types::NearToken;
use near_workspaces::{sandbox, AccountId, Worker};
use std::collections::HashMap;

mod utils;
use crate::utils::*;
use sputnik_staking::User;
use sputnikdao2::{
    default_policy, Action, BountyClaim, BountyOutput, Config, Policy, Proposal, ProposalInput,
    ProposalKind, ProposalOutput, ProposalStatus, RoleKind, RolePermission, VersionedPolicy,
    VotePolicy,
};

fn user(id: u32) -> near_sdk::AccountId {
    format!("user{}", id).parse().unwrap()
}

#[tokio::test]
async fn test_large_policy() -> Result<(), Box<dyn std::error::Error>> {
    let (sputnik_dao_factory, worker) = setup_factory().await?;

    let config = Config {
        name: "testdao".to_string(),
        purpose: "to test".to_string(),
        metadata: Base64VecU8(vec![]),
    };
    let mut policy = default_policy(vec![root()]);
    const NO_OF_COUNCILS: u32 = 10;
    const USERS_PER_COUNCIL: u32 = 100;
    for council_no in 0..NO_OF_COUNCILS {
        let mut council: Vec<near_sdk::AccountId> = vec![];
        let user_id_start = council_no * USERS_PER_COUNCIL;
        let user_id_end = user_id_start + USERS_PER_COUNCIL;
        for user_id in user_id_start..user_id_end {
            council.push(user(user_id));
        }

        let role = RolePermission {
            name: format!("council{}", council_no),
            kind: RoleKind::Group(council.into_iter().collect()),
            permissions: vec![
                "*:AddProposal".to_string(),
                "*:VoteApprove".to_string(),
                "*:VoteReject".to_string(),
                "*:VoteRemove".to_string(),
                "*:Finalize".to_string(),
            ]
            .into_iter()
            .collect(),
            vote_policy: HashMap::default(),
        };
        policy.add_or_update_role(&role);
    }

    let params = json!({ "config": config, "policy": policy })
        .to_string()
        .into_bytes();

    let create_result = sputnik_dao_factory
        .call("create")
        .args_json(json!({
            "name": "testdao",
            "args": base64::encode(params)
        }))
        .deposit(NearToken::from_near(10))
        .max_gas()
        .transact()
        .await?;

    assert!(create_result.is_success(), "{:?}", create_result.failures());

    let dao_account_id = "testdao.sputnik-dao.near";
    let dao_list = sputnik_dao_factory
        .view("get_dao_list")
        .await?
        .json::<Vec<AccountId>>()
        .unwrap();
    assert_eq!(dao_list, vec![dao_account_id]);

    Ok(())
}

#[tokio::test]
async fn test_multi_council() -> Result<(), Box<dyn std::error::Error>> {
    let (dao, _worker, root) = setup_dao().await?;
    let user1 = root
        .create_subaccount(user(1).as_str())
        .initial_balance(NearToken::from_near(1000))
        .transact()
        .await?
        .result;
    let user2 = root
        .create_subaccount(user(2).as_str())
        .initial_balance(NearToken::from_near(1000))
        .transact()
        .await?
        .result;
    let user3 = root
        .create_subaccount(user(3).as_str())
        .initial_balance(NearToken::from_near(1000))
        .transact()
        .await?
        .result;

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
                kind: RoleKind::Group(
                    vec![
                        near_sdk::AccountId::new_unchecked(user1.id().to_string()),
                        near_sdk::AccountId::new_unchecked(user2.id().to_string()),
                    ]
                    .into_iter()
                    .collect(),
                ),
                permissions: vec!["*:*".to_string()].into_iter().collect(),
                vote_policy: HashMap::default(),
            },
            RolePermission {
                name: "community".to_string(),
                kind: RoleKind::Group(
                    vec![
                        near_sdk::AccountId::new_unchecked(user1.id().to_string()),
                        near_sdk::AccountId::new_unchecked(user3.id().to_string()),
                        user(4),
                    ]
                    .into_iter()
                    .collect(),
                ),
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
    let add_proposal_result = add_proposal(
        &dao,
        ProposalInput {
            description: "new policy".to_string(),
            kind: ProposalKind::ChangePolicy {
                policy: VersionedPolicy::Current(new_policy.clone()),
            },
        },
    )
    .await;
    assert!(
        add_proposal_result.is_success(),
        "{:?}",
        add_proposal_result.failures()
    );

    assert!(vote(vec![&root], &dao, 0).await.is_ok());

    assert_eq!(
        dao.view("get_policy").await?.json::<Policy>().unwrap(),
        new_policy
    );

    let add_transfer_proposal_result = add_transfer_proposal(
        &dao,
        base_token(),
        near_sdk::AccountId::new_unchecked(user1.id().to_string()),
        1_000_000,
        None,
    )
    .await;
    assert!(
        add_transfer_proposal_result.is_success(),
        "{:?}",
        add_transfer_proposal_result.failures()
    );

    assert!(vote(vec![&user2], &dao, 1).await.is_ok());
    assert!(vote(vec![&user3], &dao, 1).await.is_ok());

    let proposal = dao
        .view("get_proposal")
        .args_json(json!({"id": 1}))
        .await?
        .json::<Proposal>()
        .unwrap();
    // Votes from members in different councils.
    assert_eq!(proposal.status, ProposalStatus::InProgress);
    // Finish with vote that is in both councils, which approves the proposal.
    assert!(vote(vec![&user1], &dao, 1).await.is_ok());

    let proposal = dao
        .view("get_proposal")
        .args_json(json!({"id": 1}))
        .await?
        .json::<Proposal>()
        .unwrap();
    assert_eq!(
        proposal.status,
        ProposalStatus::Approved,
        "{:?}",
        proposal.status
    );
    Ok(())
}

#[tokio::test]
async fn test_bounty_workflow() -> Result<(), Box<dyn std::error::Error>> {
    let (dao, worker, root) = setup_dao().await?;
    let user1 = root
        .create_subaccount(user(1).as_str())
        .initial_balance(NearToken::from_near(1000))
        .transact()
        .await?
        .result;
    let user2 = root
        .create_subaccount(user(2).as_str())
        .initial_balance(NearToken::from_near(1000))
        .transact()
        .await?
        .result;

    let proposal_id = add_bounty_proposal(&worker, &dao)
        .await
        .json::<u64>()
        .unwrap();
    assert_eq!(proposal_id, 0);
    assert_eq!(
        0,
        dao.view("get_last_proposal_id")
            .await
            .unwrap()
            .json::<u64>()
            .unwrap()
            - 1
    );

    let act_proposal_result = root
        .call(dao.id(), "act_proposal")
        .args_json(json!({"id": proposal_id, "action": Action::VoteApprove }))
        .transact()
        .await
        .unwrap();

    assert_eq!(
        0,
        act_proposal_result.failures().len(),
        "{:?}",
        act_proposal_result.failures()
    );

    let bounty_id = dao.view("get_last_bounty_id").await?.json::<u64>().unwrap() - 1;
    assert_eq!(bounty_id, 0);
    assert_eq!(
        dao.view("get_bounty")
            .args_json(json!({"id": bounty_id}))
            .await?
            .json::<BountyOutput>()
            .unwrap()
            .bounty
            .times,
        3
    );

    assert_eq!(
        NearToken::from_near(1000),
        user1.view_account().await?.balance
    );
    let bouny_claim_result = user1
        .call(dao.id(), "bounty_claim")
        .args_json(json!({
            "id": bounty_id,
            "deadline": U64::from(0)
        }))
        .deposit(NearToken::from_near(1))
        .transact()
        .await?;

    assert_eq!(
        0,
        bouny_claim_result.failures().len(),
        "{:?}",
        bouny_claim_result.failures()
    );

    assert!(
        user1.view_account().await?.balance < NearToken::from_near(999),
        "user 1 balance after bounty claim: {:?} NEAR",
        user1.view_account().await?.balance.as_near()
    );

    assert_eq!(
        1,
        dao.view("get_bounty_claims")
            .args_json(json!({"account_id": user1.id()}))
            .await
            .unwrap()
            .json::<Vec<BountyClaim>>()
            .unwrap()
            .len()
    );
    assert_eq!(
        1,
        dao.view("get_bounty_number_of_claims")
            .args_json(json!({"id": bounty_id}))
            .await
            .unwrap()
            .json::<u64>()
            .unwrap()
    );

    assert!(user1
        .call(dao.id(), "bounty_giveup")
        .args_json(json!({
            "id": bounty_id,
            "deadline": U64::from(0)
        }))
        .transact()
        .await?
        .is_success());
    assert_eq!(
        0,
        dao.view("get_bounty_claims")
            .args_json(json!({"account_id": user1.id()}))
            .await
            .unwrap()
            .json::<Vec<BountyClaim>>()
            .unwrap()
            .len()
    );
    assert_eq!(
        0,
        dao.view("get_bounty_number_of_claims")
            .args_json(json!({"id": bounty_id}))
            .await
            .unwrap()
            .json::<u64>()
            .unwrap()
    );

    assert_eq!(
        NearToken::from_near(1000),
        user2.view_account().await?.balance
    );
    assert!(user2
        .call(dao.id(), "bounty_claim")
        .args_json(json!({
            "id": bounty_id,
            "deadline": U64(worker.view_block().await.unwrap().timestamp() + 5_000_000_000)
        }))
        .deposit(NearToken::from_near(1))
        .transact()
        .await?
        .is_success());
    assert!(
        user2.view_account().await?.balance < NearToken::from_near(999),
        "user 2 balance after bounty claim: {:?} NEAR",
        user1.view_account().await?.balance.as_near()
    );
    assert_eq!(
        1,
        dao.view("get_bounty_claims")
            .args_json(json!({"account_id": user2.id()}))
            .await
            .unwrap()
            .json::<Vec<BountyClaim>>()
            .unwrap()
            .len()
    );
    assert_eq!(
        1,
        dao.view("get_bounty_number_of_claims")
            .args_json(json!({"id": bounty_id}))
            .await
            .unwrap()
            .json::<u64>()
            .unwrap()
    );

    let bounty_done_result = user2
        .call(dao.id(), "bounty_done")
        .args_json(json!({
            "id": bounty_id,
            "description": "Bounty is done"
        }))
        .deposit(NearToken::from_near(1))
        .transact()
        .await?;

    println!("Bounty done logs: {:?}", bounty_done_result.logs());
    assert_eq!(
        0,
        bounty_done_result.failures().len(),
        "{:?}",
        bounty_done_result.failures()
    );

    assert!(
        user2.view_account().await?.balance < NearToken::from_near(998),
        "user 2 balance after bounty done: {:?} NEAR",
        user1.view_account().await?.balance.as_near()
    );

    let proposal_id = dao
        .view("get_last_proposal_id")
        .await
        .unwrap()
        .json::<u64>()
        .unwrap()
        - 1;
    assert_eq!(proposal_id, 1);
    assert_eq!(
        "bounty_done",
        dao.view("get_proposal")
            .args_json(json!({"id": proposal_id}))
            .await
            .unwrap()
            .json::<ProposalOutput>()
            .unwrap()
            .proposal
            .kind
            .to_policy_label()
    );

    let act_bounty_done_proposal_result = root
        .call(dao.id(), "act_proposal")
        .args_json(json!({
            "id": proposal_id,
            "action": Action::VoteApprove
        }))
        .max_gas()
        .transact()
        .await?;
    assert!(
        act_bounty_done_proposal_result.is_success(),
        "{:?}",
        act_bounty_done_proposal_result.failures()
    );

    assert!(
        user2.view_account().await?.balance > NearToken::from_near(999),
        "{:?}",
        user1.view_account().await?.balance.as_near()
    );
    assert_eq!(
        0,
        dao.view("get_bounty_claims")
            .args_json(json!({"account_id": user2.id()}))
            .await
            .unwrap()
            .json::<Vec<BountyClaim>>()
            .unwrap()
            .len()
    );
    assert_eq!(
        dao.view("get_bounty")
            .args_json(json!({"id": bounty_id}))
            .await?
            .json::<BountyOutput>()
            .unwrap()
            .bounty
            .times,
        2
    );

    Ok(())
}

/*
#[tokio::test]
fn test_create_dao_and_use_token() {
    let (root, dao) = setup_dao();
    let user2 = root.create_user(user(2), to_yocto("1000"));
    let user3 = root.create_user(user(3), to_yocto("1000"));
    let test_token = setup_test_token(&root);
    let staking = setup_staking(&root);

    assert!(view!(dao.get_staking_contract())
        .unwrap_json::<String>()
        .is_empty());
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
        .unwrap_json::<String>()
        .is_empty());
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
#[tokio::test]
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
 */
