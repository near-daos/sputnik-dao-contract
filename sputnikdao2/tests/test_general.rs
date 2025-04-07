use near_sdk::base64::{engine::general_purpose, Engine as _};
use near_sdk::json_types::U128;
use near_sdk::serde_json::{json, Value};

use near_workspaces::types::NearToken;
use near_workspaces::{Account, AccountId};
use sputnikdao2::action_log::ActionLog;
use std::collections::HashMap;

mod utils;
use crate::utils::*;
use sputnik_staking::User;
use sputnikdao2::{
    default_policy, Action, BountyClaim, BountyOutput, Config, Policy, ProposalInput, ProposalKind,
    ProposalOutput, ProposalStatus, RoleKind, RolePermission, VersionedPolicy, VotePolicy,
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
    let mut policy = default_policy(vec![worker.root_account().unwrap().id().clone()]);
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
            "args": general_purpose::STANDARD.encode(params)
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
                    vec![user1.id().clone(), user2.id().clone()]
                        .into_iter()
                        .collect(),
                ),
                permissions: vec!["*:*".to_string()].into_iter().collect(),
                vote_policy: HashMap::default(),
            },
            RolePermission {
                name: "community".to_string(),
                kind: RoleKind::Group(
                    vec![user1.id().clone(), user3.id().clone(), user(4)]
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

    let add_transfer_proposal_result =
        add_transfer_proposal(&dao, base_token(), user1.id().clone(), 1_000_000, None).await;
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
        .json::<ProposalOutput>()
        .unwrap()
        .proposal;
    // Votes from members in different councils.
    assert_eq!(proposal.status, ProposalStatus::InProgress);
    // Finish with vote that is in both councils, which approves the proposal.
    assert!(vote(vec![&user1], &dao, 1).await.is_ok());

    let proposal = dao
        .view("get_proposal")
        .args_json(json!({"id": 1}))
        .await?
        .json::<ProposalOutput>()
        .unwrap()
        .proposal;
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
        .args_json(json!({
            "id": proposal_id,
            "action": Action::VoteApprove,
            "proposal": get_proposal_kind(&dao, proposal_id).await
        }))
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
            "action": Action::VoteApprove,
            "proposal": get_proposal_kind(&dao, proposal_id).await
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

#[tokio::test]
async fn test_create_dao_and_use_token() -> Result<(), Box<dyn std::error::Error>> {
    let (dao, _worker, root) = setup_dao().await?;
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

    let test_token = setup_test_token(&root).await?;
    let staking = setup_staking(&root, &test_token.as_account(), &dao.as_account()).await?;

    assert!(dao
        .view("get_staking_contract")
        .await?
        .json::<String>()
        .unwrap()
        .is_empty());

    let add_member_proposal_result = add_member_proposal(&dao, user2.id().clone()).await;
    assert!(
        add_member_proposal_result.is_success(),
        "{:?}",
        add_member_proposal_result.failures()
    );
    assert_eq!(
        1,
        dao.view("get_last_proposal_id")
            .await
            .unwrap()
            .json::<u64>()
            .unwrap()
    );

    // Voting by user who is not member should fail.

    let act_proposal_result = user2
        .call(dao.id(), "act_proposal")
        .args_json(json!({
            "id": 0,
            "action": Action::VoteApprove,
            "proposal": get_proposal_kind(&dao, 0).await
        }))
        .max_gas()
        .transact()
        .await?;
    assert!(format!("{:?}", act_proposal_result.failures()).contains("ERR_PERMISSION_DENIED"));

    assert!(root
        .call(dao.id(), "act_proposal")
        .args_json(json!({
            "id": 0,
            "action": Action::VoteApprove,
            "proposal": get_proposal_kind(&dao, 0).await
        }))
        .max_gas()
        .transact()
        .await?
        .is_success());

    // voting second time should fail.
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
    assert!(
        format!("{:?}", act_proposal_result.failures()).contains("ERR_PROPOSAL_NOT_READY_FOR_VOTE"),
        "{:?}",
        act_proposal_result.failures()
    );

    // Add 3rd member.
    let add_member_proposal_result = add_member_proposal(&dao, user3.id().clone()).await;
    assert!(
        add_member_proposal_result.is_success(),
        "{:?}",
        add_member_proposal_result.failures()
    );

    assert!(vote(vec![&root, &user2], &dao, 1).await.is_ok());
    let policy = dao.view("get_policy").await?.json::<Policy>().unwrap();
    assert_eq!(policy.roles.len(), 2);

    assert_eq!(
        policy.roles[1].kind,
        RoleKind::Group(
            vec![root.id().clone(), user2.id().clone(), user3.id().clone()]
                .into_iter()
                .collect()
        )
    );

    let add_proposal_result = add_proposal(
        &dao,
        ProposalInput {
            description: "test".to_string(),
            kind: ProposalKind::SetStakingContract {
                staking_id: staking.id().clone(),
            },
        },
    )
    .await;
    assert!(
        add_proposal_result.is_success(),
        "{:?}",
        add_proposal_result.failures()
    );

    assert!(vote(vec![&user3, &user2], &dao, 2).await.is_ok());
    assert!(!dao
        .view("get_staking_contract")
        .await?
        .json::<String>()
        .unwrap()
        .is_empty());
    assert_eq!(
        dao.view("get_proposal")
            .args_json(json!({"id": 2}))
            .await?
            .json::<ProposalOutput>()
            .unwrap()
            .proposal
            .status,
        ProposalStatus::Approved
    );
    let staking_ft_total_supply = staking
        .view("ft_total_supply")
        .await?
        .json::<U128>()
        .unwrap()
        .0;
    assert_eq!(0, staking_ft_total_supply);

    let mint_result = test_token
        .call("mint")
        .args_json(json!({
            "account_id": user2.id(),
            "amount": NearToken::from_near(100)
        }))
        .transact()
        .await?;
    assert!(mint_result.is_success(), "{:?}", mint_result.failures());

    let storage_deposit_result = test_token
        .call("storage_deposit")
        .args_json(json!({
            "account_id": staking.id()
        }))
        .deposit(NearToken::from_near(1))
        .transact()
        .await?;
    assert!(
        storage_deposit_result.is_success(),
        "{:?}",
        storage_deposit_result.failures()
    );

    let storage_deposit_result = user2
        .call(staking.id(), "storage_deposit")
        .args_json(json!({}))
        .deposit(NearToken::from_near(1))
        .max_gas()
        .transact()
        .await?;
    assert!(
        storage_deposit_result.is_success(),
        "{:?}",
        storage_deposit_result.failures()
    );

    let ft_transfer_result = user2
        .call(test_token.id(), "ft_transfer_call")
        .args_json(
            json!({"receiver_id": staking.id(), "amount": NearToken::from_near(10), "msg": ""}),
        )
        .deposit(NearToken::from_yoctonear(1))
        .max_gas()
        .transact()
        .await?;
    assert_eq!(
        0,
        ft_transfer_result.failures().len(),
        "{:?}",
        ft_transfer_result.failures()
    );
    println!("{:?}", ft_transfer_result.logs());

    let staking_ft_total_supply = staking
        .view("ft_total_supply")
        .await?
        .json::<U128>()
        .unwrap()
        .0;
    assert_eq!(
        NearToken::from_near(10).as_yoctonear(),
        staking_ft_total_supply
    );
    assert_eq!(
        NearToken::from_near(10).as_yoctonear(),
        staking
            .view("ft_balance_of")
            .args_json(json!({"account_id": user2.id()}))
            .await?
            .json::<U128>()
            .unwrap()
            .0
    );

    assert_eq!(
        NearToken::from_near(90).as_yoctonear(),
        test_token
            .view("ft_balance_of")
            .args_json(json!({"account_id": user2.id()}))
            .await?
            .json::<U128>()
            .unwrap()
            .0
    );

    let withdraw_result = user2
        .call(staking.id(), "withdraw")
        .args_json(json!({"amount": NearToken::from_near(5)}))
        .max_gas()
        .transact()
        .await?;
    assert!(
        withdraw_result.is_success(),
        "{:?}",
        withdraw_result.failures()
    );

    assert_eq!(
        NearToken::from_near(95).as_yoctonear(),
        test_token
            .view("ft_balance_of")
            .args_json(json!({"account_id": user2.id()}))
            .await?
            .json::<U128>()
            .unwrap()
            .0
    );

    let delegate_result = user2
        .call(staking.id(), "delegate")
        .args_json(json!({"account_id": user2.id(), "amount": NearToken::from_near(5)}))
        .max_gas()
        .transact()
        .await?;
    assert!(
        delegate_result.is_success(),
        "{:?}",
        delegate_result.failures()
    );

    let undelegate_result = user2
        .call(staking.id(), "undelegate")
        .args_json(json!({"account_id": user2.id(), "amount": NearToken::from_near(1)}))
        .max_gas()
        .transact()
        .await?;
    assert!(
        undelegate_result.is_success(),
        "{:?}",
        undelegate_result.failures()
    );

    // should fail right after undelegation as need to wait for voting period before can delegate again.
    let delegate_result = user2
        .call(staking.id(), "delegate")
        .args_json(json!({"account_id": user2.id(), "amount": NearToken::from_near(1)}))
        .max_gas()
        .transact()
        .await?;
    assert!(
        format!("{:?}", delegate_result.failures()).contains("ERR_NOT_ENOUGH_TIME_PASSED"),
        "should fail right after undelegation as need to wait for voting period before can delegate again. {:?}",
        delegate_result.failures()
    );
    let user = staking
        .view("get_user")
        .args_json(json!({"account_id": user2.id()}))
        .await?
        .json::<User>()
        .unwrap();
    assert_eq!(
        user.delegated_amounts,
        vec![(
            user2.id().clone(),
            U128(NearToken::from_near(4).as_yoctonear())
        )]
    );

    assert_eq!(
        NearToken::from_near(4).as_yoctonear(),
        dao.view("delegation_total_supply")
            .await?
            .json::<U128>()
            .unwrap()
            .0
    );
    assert_eq!(
        NearToken::from_near(4).as_yoctonear(),
        dao.view("delegation_balance_of")
            .args_json(json!({
                "account_id": user2.id()
            }))
            .await?
            .json::<U128>()
            .unwrap()
            .0
    );

    Ok(())
}

/// Test various cases that must fail.
#[tokio::test]
async fn test_failures() -> Result<(), Box<dyn std::error::Error>> {
    let (dao, _worker, _root) = setup_dao().await.unwrap();
    let add_transfer_proposal_result = add_transfer_proposal(
        &dao,
        base_token(),
        user(1),
        1_000_000,
        Some("some".to_string()),
    )
    .await;

    assert!(
        format!("{:?}", add_transfer_proposal_result.failures()).contains("ERR_BASE_TOKEN_NO_MSG"),
        "{:?}",
        add_transfer_proposal_result.failures()
    );
    Ok(())
}

/// Test payments that fail
#[tokio::test]
async fn test_payment_failures() -> Result<(), Box<dyn std::error::Error>> {
    let (dao, _worker, root) = setup_dao().await.unwrap();
    let user1 = root
        .create_subaccount(user(1).as_str())
        .initial_balance(NearToken::from_near(1000))
        .transact()
        .await?
        .result;
    let whale = root
        .create_subaccount(user(2).as_str())
        .initial_balance(NearToken::from_near(1000))
        .transact()
        .await?
        .result;

    // Add user1

    let add_member_proposal_result = add_member_proposal(&dao, user1.id().clone()).await;
    assert!(
        add_member_proposal_result.is_success(),
        "{:?}",
        add_member_proposal_result.failures()
    );

    assert!(vote(vec![&root], &dao, 0).await.is_ok());

    // Set up fungible tokens and give 5 to the dao
    let test_token = setup_test_token(&root).await.unwrap();
    assert!(dao
        .as_account()
        .call(test_token.id(), "mint")
        .args_json(json!({
            "account_id": dao.id(),
            "amount": U128(5)
        }))
        .transact()
        .await?
        .is_success());

    assert!(user1
        .call(test_token.id(), "storage_deposit")
        .args_json(json!({
            "account_id": user1.id(),
            "registration_only": true
        }))
        .deposit(NearToken::from_near(125))
        .transact()
        .await?
        .is_success());

    // Attempt to transfer more than it has
    assert!(add_transfer_proposal(
        &dao,
        Some(test_token.id().clone()),
        user1.id().clone(),
        10,
        None,
    )
    .await
    .is_success());

    // Vote in the transfer
    assert!(vote(vec![&root, &user1], &dao, 1).await.is_ok());
    let proposal = dao
        .view("get_proposal")
        .args_json(json!({"id": 1}))
        .await?
        .json::<ProposalOutput>()
        .unwrap()
        .proposal;

    assert_eq!(proposal.status, ProposalStatus::Failed);

    assert!(whale
        .call(test_token.id(), "mint")
        .args_json(json!({"account_id": whale.id(), "amount": U128(6_000_000_000)}))
        .transact()
        .await?
        .is_success());

    let ft_transfer_result = whale
        .call(test_token.id(), "ft_transfer")
        .args_json(json!({"receiver_id": dao.id(), "amount": U128(1_000),
        "msg": "Heard you're in a pinch, let me help."}))
        .deposit(NearToken::from_yoctonear(1))
        .transact()
        .await?;
    assert!(
        ft_transfer_result.is_success(),
        "{:?}",
        ft_transfer_result.failures()
    );

    // Council member retries payment via an action
    let act_proposal_result = root
        .call(dao.id(), "act_proposal")
        .args_json(json!({
            "id": 1,
            "action": Action::Finalize,
            "memo": "Sorry! We topped up our tokens. Thanks.",
            "proposal": get_proposal_kind(&dao, 1).await
        }))
        .max_gas()
        .transact()
        .await?;
    assert!(
        act_proposal_result.is_success(),
        "{:?}",
        act_proposal_result.failures()
    );
    let proposal = dao
        .view("get_proposal")
        .args_json(json!({"id": 1}))
        .await?
        .json::<ProposalOutput>()
        .unwrap()
        .proposal;

    assert_eq!(
        proposal.status,
        ProposalStatus::Approved,
        "{:?}",
        act_proposal_result.failures()
    );

    Ok(())
}

#[tokio::test]
async fn test_actions_log() -> Result<(), Box<dyn std::error::Error>> {
    let worker = near_workspaces::sandbox().await?;
    let root = worker.root_account().unwrap();
    // initialize voting users
    let mut users = Vec::new();
    for i in 0..20 {
        let account_id = user(i); // assuming user(i) returns a String
        let created = root
            .create_subaccount(account_id.as_str())
            .initial_balance(NearToken::from_near(1))
            .transact()
            .await?
            .into_result()?; // use `into_result()` instead of `.result` for better error handling

        users.push(created);
    }

    // Now add empty accounts without transaction for time optimization
    let mut policy_accounts: Vec<AccountId> = users.iter().map(|u| u.id().clone()).collect();
    for i in 21..40 {
        policy_accounts.push(user(i));
    }
    // Setup a dao with a lot of voters
    let (dao, worker, _) = setup_dao_with_params(
        root.clone(),
        worker,
        VersionedPolicy::Default(policy_accounts),
    )
    .await?;

    let proposal_id = add_bounty_proposal(&worker, &dao)
        .await
        .json::<u64>()
        .unwrap();

    // Verify add_proposal log has been added
    let blocks_log = dao
        .view("get_proposal")
        .args_json(json!({"id": proposal_id}))
        .await
        .unwrap()
        .json::<ProposalOutput>()
        .unwrap()
        .proposal
        .last_actions_log;
    let global_actions_log = dao
        .view("get_actions_log")
        .await
        .unwrap()
        .json::<Vec<ActionLog>>()
        .unwrap();

    let action_log = global_actions_log[0].clone();
    let block_log = blocks_log.get(0).unwrap();
    assert_eq!(action_log.block_height, block_log.block_height);
    assert_eq!(global_actions_log.len(), 1);
    assert_eq!(blocks_log.len(), 1);
    assert_eq!(
        action_log,
        ActionLog::new(
            "dao.test.near".parse().unwrap(),
            proposal_id,
            Action::AddProposal,
            action_log.block_height // It is uncertain because of async block creation
        )
    );
    assert!(
        (action_log.block_height as i128
            - worker.view_block().await.unwrap().header().height() as i128)
            .abs()
            <= 1 as i128,
    );

    // Fill the actions log
    let voting_users: Vec<&Account> = users.iter().take(20).collect();
    vote(voting_users, &dao, proposal_id).await.unwrap();

    // Verify that the oldest prposal now is the voting approve from user0
    let blocks_log = dao
        .view("get_proposal")
        .args_json(json!({"id": proposal_id}))
        .await
        .unwrap()
        .json::<ProposalOutput>()
        .unwrap()
        .proposal
        .last_actions_log;

    let global_actions_log = dao
        .view("get_actions_log")
        .await
        .unwrap()
        .json::<Vec<ActionLog>>()
        .unwrap();

    let action_log = global_actions_log[0].clone();
    let block_log = blocks_log[0].clone();
    assert_eq!(action_log.block_height, block_log.block_height);
    assert_eq!(global_actions_log.len(), 20);
    assert_eq!(blocks_log.len(), 20);
    assert_eq!(
        action_log,
        ActionLog::new(
            "user0.test.near".parse().unwrap(),
            proposal_id,
            Action::VoteApprove,
            action_log.block_height, // It is uncertain because of async block creation
        )
    );
    Ok(())
}

/// Test json arguments serialization
#[tokio::test]
async fn test_deny_unknown_arguments() -> Result<(), Box<dyn std::error::Error>> {
    let (dao, worker, root) = setup_dao().await.unwrap();

    // Add bounty proposal
    let add_proposal_result = add_bounty_proposal(&worker, &dao).await;
    assert!(
        add_proposal_result.is_success(),
        "{:?}",
        add_proposal_result.failures()
    );
    let kind = &mut dao
        .view("get_proposal")
        .args_json(json!({"id": 0}))
        .await
        .unwrap()
        .json::<Value>()
        .unwrap()["kind"];
    // Check it is not possible to add high level argument
    let act_proposal_result = root
        .call(dao.id(), "act_proposal")
        .args_json(json!({
            "id": 0,
            "action": Action::VoteApprove,
            "proposal": kind,
            "fake_arg": 0
        }))
        .max_gas()
        .transact()
        .await?;
    assert!(
        format!("{:?}", act_proposal_result.failures())
            .contains("Failed to deserialize input from JSON."),
        "{:?}",
        act_proposal_result.failures()
    );

    // Check it is not possible to add unknown fields to the argument struct.
    kind["AddBounty"]["bounty"]["amount1"] = near_sdk::serde_json::Value::String("100".to_string());
    let act_proposal_result = root
        .call(dao.id(), "act_proposal")
        .args_json(json!({
            "id": 0,
            "action": Action::VoteApprove,
            "proposal": kind,
        }))
        .max_gas()
        .transact()
        .await?;
    assert!(
        format!("{:?}", act_proposal_result.failures())
            .contains("Failed to deserialize input from JSON."),
        "{:?}",
        act_proposal_result.failures()
    );
    Ok(())
}
