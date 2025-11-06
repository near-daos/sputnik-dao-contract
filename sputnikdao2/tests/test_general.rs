use near_api::types::TxExecutionStatus;
use near_sandbox::config::{DEFAULT_GENESIS_ACCOUNT, DEFAULT_GENESIS_ACCOUNT_PRIVATE_KEY};
use near_sdk::base64::{engine::general_purpose, Engine as _};
use near_sdk::json_types::U128;
use near_sdk::serde_json::{json, Value};

use near_api::{AccountId, FTBalance, Reference, Signer, Staking};
use near_api::{NearToken, W_NEAR_BALANCE};
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
    format!("user{}.{}", id, DEFAULT_GENESIS_ACCOUNT)
        .parse()
        .unwrap()
}

#[tokio::test]
async fn test_large_policy() -> Result<(), Box<dyn std::error::Error>> {
    let (ctx, sputnik_dao_factory) = setup_factory().await?;

    let config = Config {
        name: "testdao".to_string(),
        purpose: "to test".to_string(),
        metadata: vec![].into(),
    };
    let mut policy = default_policy(vec![ctx.root.clone()]);
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
        .call_function(
            "create",
            json!({
                "name": "testdao",
                "args": general_purpose::STANDARD.encode(params)
            }),
        )?
        .transaction()
        .deposit(NearToken::from_near(10))
        .max_gas()
        .with_signer(ctx.root.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;

    assert!(create_result.is_success(), "{:?}", create_result.failures());

    let dao_account_id = "testdao.sputnik-dao.near";
    let dao_list: Vec<AccountId> = sputnik_dao_factory
        .call_function("get_dao_list", json!({}))?
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(dao_list, vec![dao_account_id]);

    Ok(())
}

#[tokio::test]
async fn test_multi_council() -> Result<(), Box<dyn std::error::Error>> {
    let (ctx, dao) = setup_dao().await?;
    near_api::Account::create_account(user(1))
        .fund_myself(ctx.root.clone(), NearToken::from_near(1000))
        .public_key(ctx.signer.get_public_key().await?)?
        .with_signer(ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .assert_success();
    near_api::Account::create_account(user(2))
        .fund_myself(ctx.root.clone(), NearToken::from_near(1000))
        .public_key(ctx.signer.get_public_key().await?)?
        .with_signer(ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .assert_success();
    near_api::Account::create_account(user(3))
        .fund_myself(ctx.root.clone(), NearToken::from_near(1000))
        .public_key(ctx.signer.get_public_key().await?)?
        .with_signer(ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .assert_success();

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
                kind: RoleKind::Group(
                    vec![user(1), user(3), user(4).clone()]
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
        &ctx,
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

    assert!(vote(&ctx, vec![&ctx.root], &dao, 0).await.is_ok());

    assert_eq!(
        dao.call_function("get_policy", json!({}))?
            .read_only::<Policy>()
            .fetch_from(&ctx.sandbox_network)
            .await?
            .data,
        new_policy
    );

    let add_transfer_proposal_result =
        add_transfer_proposal(&ctx, &dao, base_token(), user(1), 1_000_000, None).await;
    assert!(
        add_transfer_proposal_result.is_success(),
        "{:?}",
        add_transfer_proposal_result.failures()
    );

    assert!(vote(&ctx, vec![&user(2)], &dao, 1).await.is_ok());
    assert!(vote(&ctx, vec![&user(3)], &dao, 1).await.is_ok());

    let proposal = dao
        .call_function("get_proposal", json!({"id": 1}))?
        .read_only::<ProposalOutput>()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data
        .proposal;
    // Votes from members in different councils.
    assert_eq!(proposal.status, ProposalStatus::InProgress);
    // Finish with vote that is in both councils, which approves the proposal.
    assert!(vote(&ctx, vec![&user(1)], &dao, 1).await.is_ok());

    let proposal = dao
        .call_function("get_proposal", json!({"id": 1}))?
        .read_only::<ProposalOutput>()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data
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
    let (ctx, dao) = setup_dao().await?;
    near_api::Account::create_account(user(1))
        .fund_myself(ctx.root.clone(), NearToken::from_near(1000))
        .public_key(ctx.signer.get_public_key().await?)?
        .with_signer(ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .assert_success();
    near_api::Account::create_account(user(2))
        .fund_myself(ctx.root.clone(), NearToken::from_near(1000))
        .public_key(ctx.signer.get_public_key().await?)?
        .with_signer(ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .assert_success();

    let proposal_id: u64 = add_bounty_proposal(&ctx, &dao)
        .await
        .assert_success()
        .json()?;
    assert_eq!(proposal_id, 0);
    let last_proposal_id: u64 = dao
        .call_function("get_last_proposal_id", json!({}))?
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(0, last_proposal_id - 1);

    let act_proposal_result = dao
        .call_function(
            "act_proposal",
            json!({
                "id": proposal_id,
                "action": Action::VoteApprove,
                "proposal": get_proposal_kind(&ctx, &dao, proposal_id).await
            }),
        )?
        .transaction()
        .with_signer(ctx.root.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;

    assert_eq!(
        0,
        act_proposal_result.failures().len(),
        "{:?}",
        act_proposal_result.failures()
    );

    let bounty_id = dao
        .call_function("get_last_bounty_id", json!({}))?
        .read_only::<u64>()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data
        - 1;
    assert_eq!(bounty_id, 0);

    let bounty: BountyOutput = dao
        .call_function("get_bounty", json!({"id": bounty_id}))?
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(bounty.bounty.times, 3);

    assert_eq!(
        NearToken::from_near(1000),
        near_api::Tokens::account(user(1))
            .near_balance()
            .fetch_from(&ctx.sandbox_network)
            .await?
            .total
    );

    let bounty_claim_result = dao
        .call_function(
            "bounty_claim",
            json!({
                "id": bounty_id,
                "deadline": U64::from(0)
            }),
        )?
        .transaction()
        .deposit(NearToken::from_near(1))
        .with_signer(user(1), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;

    assert_eq!(
        0,
        bounty_claim_result.failures().len(),
        "{:?}",
        bounty_claim_result.failures()
    );

    let user_balance = near_api::Tokens::account(user(1))
        .near_balance()
        .fetch_from(&ctx.sandbox_network)
        .await?;
    assert!(
        user_balance.total < NearToken::from_near(999),
        "user 1 balance after bounty claim: {}",
        user_balance.total
    );

    assert_eq!(
        1,
        dao.call_function("get_bounty_claims", json!({"account_id": user(1)}))?
            .read_only::<Vec<BountyClaim>>()
            .fetch_from(&ctx.sandbox_network)
            .await?
            .data
            .len()
    );
    assert_eq!(
        1,
        dao.call_function("get_bounty_number_of_claims", json!({"id": bounty_id}))?
            .read_only::<u64>()
            .fetch_from(&ctx.sandbox_network)
            .await?
            .data
    );

    dao.call_function(
        "bounty_giveup",
        json!({
            "id": bounty_id,
            "deadline": U64::from(0)
        }),
    )?
    .transaction()
    .with_signer(user(1), ctx.signer.clone())
    .send_to(&ctx.sandbox_network)
    .await?
    .assert_success();

    assert_eq!(
        0,
        dao.call_function("get_bounty_claims", json!({"account_id": user(1)}))?
            .read_only::<Vec<BountyClaim>>()
            .fetch_from(&ctx.sandbox_network)
            .await?
            .data
            .len()
    );

    assert_eq!(
        0,
        dao.call_function("get_bounty_number_of_claims", json!({"id": bounty_id}))?
            .read_only::<u64>()
            .fetch_from(&ctx.sandbox_network)
            .await?
            .data
    );

    assert_eq!(
        NearToken::from_near(1000),
        near_api::Tokens::account(user(2))
            .near_balance()
            .fetch_from(&ctx.sandbox_network)
            .await?
            .total,
    );
    let block_timestamp = near_api::Chain::block()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .header
        .timestamp;
    dao.call_function(
        "bounty_claim",
        json!({
            "id": bounty_id,
            "deadline": U64(block_timestamp + 5_000_000_000)
        }),
    )?
    .transaction()
    .deposit(NearToken::from_near(1))
    .with_signer(user(2), ctx.signer.clone())
    .send_to(&ctx.sandbox_network)
    .await?
    .assert_success();

    let user2_balance = near_api::Tokens::account(user(2))
        .near_balance()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .total;
    assert!(
        user2_balance < NearToken::from_near(999),
        "user 2 balance after bounty claim: {}",
        user2_balance
    );

    assert_eq!(
        1,
        dao.call_function("get_bounty_claims", json!({"account_id": user(2)}))?
            .read_only::<Vec<BountyClaim>>()
            .fetch_from(&ctx.sandbox_network)
            .await?
            .data
            .len()
    );
    assert_eq!(
        1,
        dao.call_function("get_bounty_number_of_claims", json!({"id": bounty_id}))?
            .read_only::<u64>()
            .fetch_from(&ctx.sandbox_network)
            .await?
            .data
    );
    let bounty_done_result = dao
        .call_function(
            "bounty_done",
            json!({
                "id": bounty_id,
                "description": "Bounty is done"
            }),
        )?
        .transaction()
        .deposit(NearToken::from_near(1))
        .with_signer(user(2), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;

    println!("Bounty done logs: {:?}", bounty_done_result.logs());
    assert_eq!(
        0,
        bounty_done_result.failures().len(),
        "{:?}",
        bounty_done_result.failures()
    );

    let user2_balance = near_api::Tokens::account(user(2))
        .near_balance()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .total;
    assert!(
        user2_balance < NearToken::from_near(998),
        "user 2 balance after bounty done: {}",
        user2_balance
    );

    let proposal_id: u64 = dao
        .call_function("get_last_proposal_id", json!({}))?
        .read_only::<u64>()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data
        - 1;
    assert_eq!(proposal_id, 1);
    assert_eq!(
        "bounty_done",
        dao.call_function("get_proposal", json!({"id": proposal_id}))?
            .read_only::<ProposalOutput>()
            .fetch_from(&ctx.sandbox_network)
            .await?
            .data
            .proposal
            .kind
            .to_policy_label()
    );

    let act_bounty_done_proposal_result = dao
        .call_function(
            "act_proposal",
            json!({
                "id": proposal_id,
                "action": Action::VoteApprove,
                "proposal": get_proposal_kind(&ctx, &dao, proposal_id).await
            }),
        )?
        .transaction()
        .max_gas()
        .with_signer(ctx.root.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        act_bounty_done_proposal_result.is_success(),
        "{:?}",
        act_bounty_done_proposal_result.failures()
    );

    let user2_balance = near_api::Tokens::account(user(2))
        .near_balance()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .total;
    assert!(
        user2_balance > NearToken::from_near(999),
        "{}",
        user2_balance
    );
    assert_eq!(
        0,
        dao.call_function("get_bounty_claims", json!({"account_id": user(2)}))?
            .read_only::<Vec<BountyClaim>>()
            .fetch_from(&ctx.sandbox_network)
            .await?
            .data
            .len()
    );
    assert_eq!(
        2,
        dao.call_function("get_bounty", json!({"id": bounty_id}))?
            .read_only::<BountyOutput>()
            .fetch_from(&ctx.sandbox_network)
            .await?
            .data
            .bounty
            .times,
    );

    Ok(())
}

#[tokio::test]
async fn test_create_dao_and_use_token() -> Result<(), Box<dyn std::error::Error>> {
    let (ctx, dao) = setup_dao().await?;
    near_api::Account::create_account(user(2))
        .fund_myself(ctx.root.clone(), NearToken::from_near(1000))
        .public_key(ctx.signer.get_public_key().await?)?
        .with_signer(ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .assert_success();
    near_api::Account::create_account(user(3))
        .fund_myself(ctx.root.clone(), NearToken::from_near(1000))
        .public_key(ctx.signer.get_public_key().await?)?
        .with_signer(ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .assert_success();

    let test_token = setup_test_token(&ctx).await?;
    let staking = setup_staking(&ctx, &test_token.0, &dao.0).await?;

    assert!(dao
        .call_function("get_staking_contract", json!({}))?
        .read_only::<String>()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data
        .is_empty());

    let add_member_proposal_result = add_member_proposal(&ctx, &dao, user(2)).await;
    assert!(
        add_member_proposal_result.is_success(),
        "{:?}",
        add_member_proposal_result.failures()
    );
    assert_eq!(
        1,
        dao.call_function("get_last_proposal_id", json!({}))?
            .read_only::<u64>()
            .fetch_from(&ctx.sandbox_network)
            .await?
            .data
    );

    // Voting by user who is not member should fail.

    let act_proposal_result = dao
        .call_function(
            "act_proposal",
            json!({
                "id": 0,
                "action": Action::VoteApprove,
                "proposal": get_proposal_kind(&ctx, &dao, 0).await
            }),
        )?
        .transaction()
        .max_gas()
        .with_signer(user(2), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(format!("{:?}", act_proposal_result.failures()).contains("ERR_PERMISSION_DENIED"));

    dao.call_function(
        "act_proposal",
        json!({
            "id": 0,
            "action": Action::VoteApprove,
            "proposal": get_proposal_kind(&ctx, &dao, 0).await
        }),
    )?
    .transaction()
    .max_gas()
    .with_signer(ctx.root.clone(), ctx.signer.clone())
    .send_to(&ctx.sandbox_network)
    .await?
    .assert_success();

    // voting second time should fail.
    let act_proposal_result = dao
        .call_function(
            "act_proposal",
            json!({
                "id": 0,
                "action": Action::VoteApprove,
                "proposal": get_proposal_kind(&ctx, &dao, 0).await
            }),
        )?
        .transaction()
        .max_gas()
        .with_signer(ctx.root.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        format!("{:?}", act_proposal_result.failures()).contains("ERR_PROPOSAL_NOT_READY_FOR_VOTE"),
        "{:?}",
        act_proposal_result.failures()
    );

    // Add 3rd member.
    let add_member_proposal_result = add_member_proposal(&ctx, &dao, user(3)).await;
    assert!(
        add_member_proposal_result.is_success(),
        "{:?}",
        add_member_proposal_result.failures()
    );

    assert!(vote(&ctx, vec![&ctx.root, &user(2)], &dao, 1).await.is_ok());
    let policy = dao
        .call_function("get_policy", json!({}))?
        .read_only::<Policy>()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(policy.roles.len(), 2);

    assert_eq!(
        policy.roles[1].kind,
        RoleKind::Group(
            vec![ctx.root.clone(), user(2), user(3)]
                .into_iter()
                .collect()
        )
    );

    let add_proposal_result = add_proposal(
        &ctx,
        &dao,
        ProposalInput {
            description: "test".to_string(),
            kind: ProposalKind::SetStakingContract {
                staking_id: staking.0.clone(),
            },
        },
    )
    .await;
    assert!(
        add_proposal_result.is_success(),
        "{:?}",
        add_proposal_result.failures()
    );

    assert!(vote(&ctx, vec![&user(3), &user(2)], &dao, 2).await.is_ok());
    assert!(!dao
        .call_function("get_staking_contract", json!({}))?
        .read_only::<String>()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data
        .is_empty());
    assert_eq!(
        dao.call_function("get_proposal", json!({"id": 2}))?
            .read_only::<ProposalOutput>()
            .fetch_from(&ctx.sandbox_network)
            .await?
            .data
            .proposal
            .status,
        ProposalStatus::Approved
    );

    let staking_ft_total_supply = staking
        .call_function("ft_total_supply", json!({}))?
        .read_only::<U128>()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(0, staking_ft_total_supply.0);

    let mint_result = test_token
        .call_function(
            "mint",
            json!({
                "account_id": user(2),
                "amount": NearToken::from_near(100)
            }),
        )?
        .transaction()
        .with_signer(ctx.root.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(mint_result.is_success(), "{:?}", mint_result.failures());

    let storage_deposit_result = near_api::StorageDeposit::on_contract(staking.0.clone())
        .deposit(user(2), NearToken::from_near(1))?
        .with_signer(user(2), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        storage_deposit_result.is_success(),
        "{:?}",
        storage_deposit_result.failures()
    );

    let ft_transfer_result = near_api::Tokens::account(user(2))
        .send_to(staking.0.clone())
        .ft_call(
            test_token.0.clone(),
            W_NEAR_BALANCE.with_whole_amount(10),
            "".to_string(),
        )?
        .with_signer(ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;

    assert_eq!(
        0,
        ft_transfer_result.failures().len(),
        "{:?}",
        ft_transfer_result.failures()
    );
    println!("{:?}", ft_transfer_result.logs());

    let staking_ft_total_supply = staking
        .call_function("ft_total_supply", json!({}))?
        .read_only::<U128>()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(
        NearToken::from_near(10).as_yoctonear(),
        staking_ft_total_supply.0
    );
    assert_eq!(
        NearToken::from_near(10).as_yoctonear(),
        staking
            .call_function("ft_balance_of", json!({"account_id": user(2)}))?
            .read_only::<U128>()
            .fetch_from(&ctx.sandbox_network)
            .await?
            .data
            .0
    );

    assert_eq!(
        NearToken::from_near(90).as_yoctonear(),
        test_token
            .call_function("ft_balance_of", json!({"account_id": user(2)}))?
            .read_only::<U128>()
            .fetch_from(&ctx.sandbox_network)
            .await?
            .data
            .0
    );

    Staking::delegation(user(2))
        .withdraw(staking.0.clone(), NearToken::from_near(5))?
        .with_signer(ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .assert_success();

    assert_eq!(
        NearToken::from_near(95).as_yoctonear(),
        test_token
            .call_function("ft_balance_of", json!({"account_id": user(2)}))?
            .read_only::<U128>()
            .fetch_from(&ctx.sandbox_network)
            .await?
            .data
            .0
    );

    let delegate_result = staking
        .call_function(
            "delegate",
            json!({"account_id": user(2), "amount": NearToken::from_near(5)}),
        )?
        .transaction()
        .max_gas()
        .with_signer(user(2), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        delegate_result.is_success(),
        "{:?}",
        delegate_result.failures()
    );

    let undelegate_result = staking
        .call_function(
            "undelegate",
            json!({"account_id": user(2), "amount": NearToken::from_near(1)}),
        )?
        .transaction()
        .max_gas()
        .with_signer(user(2), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        undelegate_result.is_success(),
        "{:?}",
        undelegate_result.failures()
    );

    // should fail right after undelegation as need to wait for voting period before can delegate again.
    let delegate_result = staking
        .call_function(
            "delegate",
            json!({"account_id": user(2), "amount": NearToken::from_near(1)}),
        )?
        .transaction()
        .max_gas()
        .with_signer(user(2), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        format!("{:?}", delegate_result.failures()).contains("ERR_NOT_ENOUGH_TIME_PASSED"),
        "should fail right after undelegation as need to wait for voting period before can delegate again. {:?}",
        delegate_result.failures()
    );
    let user_info = staking
        .call_function("get_user", json!({"account_id": user(2)}))?
        .read_only::<User>()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(
        user_info.delegated_amounts,
        vec![(user(2), NearToken::from_near(4).as_yoctonear().into())]
    );

    assert_eq!(
        NearToken::from_near(4).as_yoctonear(),
        dao.call_function("delegation_total_supply", json!({}))?
            .read_only::<U128>()
            .fetch_from(&ctx.sandbox_network)
            .await?
            .data
            .0
    );
    assert_eq!(
        NearToken::from_near(4).as_yoctonear(),
        dao.call_function("delegation_balance_of", json!({"account_id": user(2)}))?
            .read_only::<U128>()
            .fetch_from(&ctx.sandbox_network)
            .await?
            .data
            .0
    );

    Ok(())
}

/// Test various cases that must fail.
#[tokio::test]
async fn test_failures() -> Result<(), Box<dyn std::error::Error>> {
    let (ctx, dao) = setup_dao().await?;
    let add_transfer_proposal_result = add_transfer_proposal(
        &ctx,
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
    let (ctx, dao) = setup_dao().await?;
    let (user1, whale) = (user(1), user(2));

    near_api::Account::create_account(user(1))
        .fund_myself(ctx.root.clone(), NearToken::from_near(1000))
        .public_key(ctx.signer.get_public_key().await?)?
        .with_signer(ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .assert_success();
    near_api::Account::create_account(user(2))
        .fund_myself(ctx.root.clone(), NearToken::from_near(1000))
        .public_key(ctx.signer.get_public_key().await?)?
        .with_signer(ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .assert_success();

    // Add user1

    let add_member_proposal_result = add_member_proposal(&ctx, &dao, user1.clone()).await;
    assert!(
        add_member_proposal_result.is_success(),
        "{:?}",
        add_member_proposal_result.failures()
    );

    assert!(vote(&ctx, vec![&ctx.root], &dao, 0).await.is_ok());

    // Set up fungible tokens and give 5 to the dao
    let test_token = setup_test_token(&ctx).await?;
    test_token
        .call_function(
            "mint",
            json!({
                "account_id": dao.0.clone(),
                "amount": U128(5)
            }),
        )?
        .transaction()
        .with_signer(dao.0.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .assert_success();

    near_api::StorageDeposit::on_contract(test_token.0.clone())
        .deposit(user1.clone(), NearToken::from_near(1))?
        .with_signer(user1.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .assert_success();

    // Attempt to transfer more than it has
    assert!(add_transfer_proposal(
        &ctx,
        &dao,
        Some(test_token.0.clone()),
        user1.clone(),
        10,
        None,
    )
    .await
    .is_success());

    // Vote in the transfer
    assert!(vote(&ctx, vec![&ctx.root.clone(), &user1], &dao, 1)
        .await
        .is_ok());
    let proposal = dao
        .call_function("get_proposal", json!({"id": 1}))?
        .read_only::<ProposalOutput>()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data
        .proposal;

    assert_eq!(proposal.status, ProposalStatus::Failed);

    test_token
        .call_function(
            "mint",
            json!({"account_id": whale.clone(), "amount": U128(6_000_000_000)}),
        )?
        .transaction()
        .with_signer(whale.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .assert_success();

    let ft_transfer_result = near_api::Tokens::account(whale.clone())
        .send_to(dao.0.clone())
        .ft(
            test_token.0.clone(),
            FTBalance::with_decimals(0).with_amount(1000),
        )?
        .with_signer(ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        ft_transfer_result.is_success(),
        "{:?}",
        ft_transfer_result.failures()
    );

    // Council member retries payment via an action
    let act_proposal_result = dao
        .call_function(
            "act_proposal",
            json!({
                "id": 1,
                "action": Action::Finalize,
                "memo": "Sorry! We topped up our tokens. Thanks.",
                "proposal": get_proposal_kind(&ctx, &dao, 1).await
            }),
        )?
        .transaction()
        .max_gas()
        .with_signer(ctx.root.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        act_proposal_result.is_success(),
        "{:?}",
        act_proposal_result.failures()
    );
    let proposal = dao
        .call_function("get_proposal", json!({"id": 1}))?
        .read_only::<ProposalOutput>()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data
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
    let sandbox = near_sandbox::Sandbox::start_sandbox().await?;
    let sandbox_network =
        near_api::NetworkConfig::from_rpc_url("sandbox", sandbox.rpc_addr.parse()?);
    let root = DEFAULT_GENESIS_ACCOUNT.to_owned();
    let signer = Signer::new(Signer::from_secret_key(
        DEFAULT_GENESIS_ACCOUNT_PRIVATE_KEY.parse()?,
    ))?;
    // initialize voting users
    let mut users = Vec::new();
    for i in 0..20 {
        let account_id = user(i); // assuming user(i) returns a String
        near_api::Account::create_account(account_id.clone())
            .fund_myself(root.clone(), NearToken::from_near(1))
            .public_key(signer.get_public_key().await?)?
            .with_signer(signer.clone())
            .wait_until(TxExecutionStatus::ExecutedOptimistic)
            .send_to(&sandbox_network)
            .await?
            .assert_success();

        users.push(account_id);
    }

    // Now add empty accounts without transaction for time optimization
    let mut policy_accounts: Vec<AccountId> = users.to_vec();
    for i in 21..40 {
        policy_accounts.push(user(i));
    }
    // Setup a dao with a lot of voters
    let (ctx, dao) = setup_dao_with_params(
        root.clone(),
        signer,
        sandbox,
        VersionedPolicy::Default(policy_accounts),
    )
    .await?;

    let proposal_id = add_bounty_proposal(&ctx, &dao).await.json::<u64>()?;

    // Verify add_proposal log has been added
    let proposal_log = dao
        .call_function("get_proposal", json!({"id": proposal_id}))?
        .read_only::<ProposalOutput>()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data
        .proposal
        .last_actions_log;
    let global_actions_log = dao
        .call_function("get_actions_log", json!({}))?
        .read_only::<Vec<ActionLog>>()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;

    let action_log = global_actions_log[0].clone();
    let block_log = proposal_log.front().unwrap();
    assert_eq!(action_log.block_height, block_log.block_height);
    assert_eq!(global_actions_log.len(), 1);
    assert_eq!(proposal_log.len(), 1);
    assert_eq!(
        action_log,
        ActionLog {
            account_id: "dao.sandbox".parse()?,
            proposal_id: proposal_id.into(),
            action: Action::AddProposal,
            block_height: action_log.block_height // It is uncertain because of async block creation
        }
    );
    let block_height = near_api::Chain::block_number()
        .at(Reference::Final)
        .fetch_from(&ctx.sandbox_network)
        .await?;
    assert!((action_log.block_height.0 as i128 - block_height as i128).abs() <= 1_i128);

    // Fill the actions log
    let voting_users: Vec<&AccountId> = users.iter().take(20).collect();
    vote(&ctx, voting_users, &dao, proposal_id).await?;

    // Verify that the oldest prposal now is the voting approve from user0
    let proposal_log = dao
        .call_function("get_proposal", json!({"id": proposal_id}))?
        .read_only::<ProposalOutput>()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data
        .proposal
        .last_actions_log;

    let global_actions_log = dao
        .call_function("get_actions_log", json!({}))?
        .read_only::<Vec<ActionLog>>()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;

    let action_log = global_actions_log[0].clone();
    let block_log = proposal_log[0].clone();
    assert_eq!(action_log.block_height, block_log.block_height);
    assert_eq!(global_actions_log.len(), 20);
    assert_eq!(proposal_log.len(), 20);
    assert_eq!(
        action_log,
        ActionLog {
            account_id: "user0.sandbox".parse()?,
            proposal_id: proposal_id.into(),
            action: Action::VoteApprove,
            block_height: action_log.block_height, // It is uncertain because of async block creation
        }
    );
    Ok(())
}

/// Test json arguments serialization
#[tokio::test]
async fn test_deny_unknown_arguments() -> Result<(), Box<dyn std::error::Error>> {
    let (ctx, dao) = setup_dao().await?;

    // Add bounty proposal
    let add_proposal_result = add_bounty_proposal(&ctx, &dao).await;
    assert!(
        add_proposal_result.is_success(),
        "{:?}",
        add_proposal_result.failures()
    );
    let mut kind = dao
        .call_function("get_proposal", json!({"id": 0}))?
        .read_only::<Value>()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data["kind"]
        .clone();
    // Check it is not possible to add high level argument
    let act_proposal_result = dao
        .clone()
        .call_function(
            "act_proposal",
            json!({
                "id": 0,
                "action": Action::VoteApprove,
                "proposal": kind,
                "fake_arg": 0
            }),
        )?
        .transaction()
        .max_gas()
        .with_signer(ctx.root.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        format!("{:?}", act_proposal_result.failures())
            .contains("Failed to deserialize input from JSON."),
        "{:?}",
        act_proposal_result.failures()
    );

    // Check it is not possible to add unknown fields to the argument struct.
    kind["AddBounty"]["bounty"]["amount1"] = near_sdk::serde_json::Value::String("100".to_string());
    let act_proposal_result = dao
        .clone()
        .call_function(
            "act_proposal",
            json!({
                "id": 0,
                "action": Action::VoteApprove,
                "proposal": kind,
            }),
        )?
        .transaction()
        .max_gas()
        .with_signer(ctx.root.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        format!("{:?}", act_proposal_result.failures())
            .contains("Failed to deserialize input from JSON."),
        "{:?}",
        act_proposal_result.failures()
    );
    Ok(())
}
