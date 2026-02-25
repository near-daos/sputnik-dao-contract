use near_sdk::base64::{Engine as _, engine::general_purpose};
use near_sdk::json_types::{Base64VecU8, U64, U128};
use near_sdk::serde_json::json;
use std::collections::HashMap;

use near_api::NearToken;
use sputnikdao2::{
    Action, Config, Policy, ProposalInput, ProposalKind, ProposalOutput, ProposalStatus, RoleKind,
    RolePermission, VersionedPolicy, VotePolicy,
};

mod utils;
use crate::utils::*;

#[tokio::test]
async fn test_add_proposal_insufficient_deposit() -> testresult::TestResult {
    let (ctx, dao) = setup_dao().await?;
    let alice = create_named_account(&ctx, "alice", 100).await?;

    // Initial last proposal id is 0
    let last_id: u64 = dao
        .call_function("get_last_proposal_id", json!({}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(last_id, 0);

    let config = Config {
        name: "sputnikdao".to_string(),
        purpose: "testing".to_string(),
        metadata: Base64VecU8(vec![]),
    };

    // Try with slightly less than 1 NEAR — should fail with ERR_MIN_BOND
    let result = dao
        .call_function(
            "add_proposal",
            json!({
                "proposal": {
                    "description": "rename the dao",
                    "kind": { "ChangeConfig": { "config": config } }
                }
            }),
        )
        .transaction()
        .deposit(NearToken::from_yoctonear(
            NearToken::from_near(1).as_yoctonear() - 1,
        ))
        .with_signer(alice.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        format!("{:?}", result.failures()).contains("ERR_MIN_BOND"),
        "{:?}",
        result.failures()
    );

    // Proposal count must still be 0
    let last_id: u64 = dao
        .call_function("get_last_proposal_id", json!({}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(last_id, 0);

    // Now with exactly 1 NEAR — should succeed
    dao.call_function(
        "add_proposal",
        json!({
            "proposal": {
                "description": "rename the dao",
                "kind": { "ChangeConfig": { "config": config } }
            }
        }),
    )
    .transaction()
    .deposit(NearToken::from_near(1))
    .with_signer(alice.clone(), ctx.signer.clone())
    .send_to(&ctx.sandbox_network)
    .await?
    .into_result()?;

    let last_id: u64 = dao
        .call_function("get_last_proposal_id", json!({}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(last_id, 1);

    // Check proposal contents
    let proposal_output: ProposalOutput = dao
        .call_function("get_proposal", json!({"id": 0}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(proposal_output.proposal.description, "rename the dao");
    assert_eq!(proposal_output.proposal.proposer, alice);
    assert_eq!(proposal_output.proposal.status, ProposalStatus::InProgress);

    Ok(())
}

#[tokio::test]
async fn test_bob_cannot_add_proposals() -> testresult::TestResult {
    let (ctx, dao) = setup_dao().await?;
    let alice = create_named_account(&ctx, "alice", 100).await?;
    let bob = create_named_account(&ctx, "bob", 100).await?;

    let period = U64::from(1_000_000_000u64 * 60 * 60 * 24 * 7);

    let new_policy = Policy {
        roles: vec![RolePermission {
            name: "all".to_string(),
            kind: RoleKind::Group(vec![ctx.root.clone(), alice.clone()].into_iter().collect()),
            permissions: vec!["*:VoteApprove".to_string(), "*:AddProposal".to_string()]
                .into_iter()
                .collect(),
            vote_policy: HashMap::default(),
        }],
        default_vote_policy: VotePolicy::default(),
        proposal_bond: NearToken::from_near(1),
        proposal_period: period,
        bounty_bond: NearToken::from_near(1),
        bounty_forgiveness_period: period,
    };

    // Bob adds a ChangePolicy proposal (everyone can add proposals initially)
    let proposal_id: u64 = add_proposal_as(
        &ctx,
        &dao,
        &bob,
        ProposalInput {
            description: "change policy so bob can't add proposals".to_string(),
            kind: ProposalKind::ChangePolicy {
                policy: VersionedPolicy::Current(new_policy),
            },
        },
    )
    .await
    .json()?;

    // Root votes to approve
    vote(&ctx, vec![&ctx.root], &dao, proposal_id).await?;

    // Bob tries to add another proposal — should now be denied
    let result = add_proposal_as(
        &ctx,
        &dao,
        &bob,
        ProposalInput {
            description: "change policy".to_string(),
            kind: ProposalKind::ChangePolicy {
                policy: VersionedPolicy::Current(Policy {
                    roles: vec![],
                    default_vote_policy: VotePolicy::default(),
                    proposal_bond: NearToken::from_near(1),
                    proposal_period: period,
                    bounty_bond: NearToken::from_near(1),
                    bounty_forgiveness_period: period,
                }),
            },
        },
    )
    .await;
    assert!(
        format!("{:?}", result.failures()).contains("ERR_PERMISSION_DENIED"),
        "{:?}",
        result.failures()
    );

    Ok(())
}

#[tokio::test]
async fn test_proposal_change_policy() -> testresult::TestResult {
    let (ctx, dao) = setup_dao().await?;
    let alice = create_named_account(&ctx, "alice", 100).await?;

    // get_proposals returns empty initially
    let proposals: Vec<ProposalOutput> = dao
        .call_function("get_proposals", json!({"from_index": 0, "limit": 10}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert!(proposals.is_empty());

    // Invalid policy (just a vec of account IDs) should be rejected
    let result = dao
        .call_function(
            "add_proposal",
            json!({
                "proposal": {
                    "description": "change the policy",
                    "kind": {
                        "ChangePolicy": {
                            "policy": [ctx.root.clone()]  // not a valid VersionedPolicy::Current
                        }
                    }
                }
            }),
        )
        .transaction()
        .deposit(NearToken::from_near(1))
        .with_signer(alice.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        format!("{:?}", result.failures()).contains("ERR_INVALID_POLICY"),
        "{:?}",
        result.failures()
    );

    let period = U64::from(1_000_000_000u64 * 60 * 60 * 24 * 7);
    let correct_policy = Policy {
        roles: vec![RolePermission {
            name: "all".to_string(),
            kind: RoleKind::Group(vec![ctx.root.clone(), alice.clone()].into_iter().collect()),
            permissions: vec!["*:AddProposal".to_string(), "*:VoteApprove".to_string()]
                .into_iter()
                .collect(),
            vote_policy: HashMap::default(),
        }],
        default_vote_policy: VotePolicy::default(),
        proposal_bond: NearToken::from_near(1),
        proposal_period: period,
        bounty_bond: NearToken::from_near(1),
        bounty_forgiveness_period: period,
    };

    let proposal_id: u64 = add_proposal_as(
        &ctx,
        &dao,
        &alice,
        ProposalInput {
            description: "change to a new correct policy".to_string(),
            kind: ProposalKind::ChangePolicy {
                policy: VersionedPolicy::Current(correct_policy.clone()),
            },
        },
    )
    .await
    .json()?;

    // last_proposal_id should now be 1
    let last_id: u64 = dao
        .call_function("get_last_proposal_id", json!({}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(last_id, 1);

    // Vote to approve
    vote(&ctx, vec![&ctx.root], &dao, proposal_id).await?;

    // Proposal should now be Approved
    let proposal_output: ProposalOutput = dao
        .call_function("get_proposal", json!({"id": 0}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(proposal_output.proposal.status, ProposalStatus::Approved);

    // Policy should have changed
    let actual_policy: Policy = dao
        .call_function("get_policy", json!({}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(actual_policy, correct_policy);

    Ok(())
}

#[tokio::test]
async fn test_proposal_transfer() -> testresult::TestResult {
    let (ctx, dao) = setup_dao().await?;
    let alice = create_named_account(&ctx, "alice", 100).await?;

    // Transfer with empty token_id and a msg should fail: ERR_BASE_TOKEN_NO_MSG
    let result = dao
        .call_function(
            "add_proposal",
            json!({
                "proposal": {
                    "description": "should fail",
                    "kind": {
                        "Transfer": {
                            "token_id": "",
                            "receiver_id": alice,
                            "amount": U128(NearToken::from_near(1).as_yoctonear()),
                            "msg": "some msg"
                        }
                    }
                }
            }),
        )
        .transaction()
        .deposit(NearToken::from_near(1))
        .with_signer(ctx.root.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        format!("{:?}", result.failures()).contains("ERR_BASE_TOKEN_NO_MSG"),
        "{:?}",
        result.failures()
    );

    // Valid transfer of 1 NEAR to alice
    let alice_balance_before = near_api::Tokens::account(alice.clone())
        .near_balance()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .total;

    let transfer_id: u64 = add_transfer_proposal(
        &ctx,
        &dao,
        base_token(),
        alice.clone(),
        NearToken::from_near(1).as_yoctonear(),
        None,
    )
    .await
    .json()?;

    vote(&ctx, vec![&ctx.root], &dao, transfer_id).await?;

    let proposal: ProposalOutput = dao
        .call_function("get_proposal", json!({"id": transfer_id}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(proposal.proposal.status, ProposalStatus::Approved);

    let alice_balance_after = near_api::Tokens::account(alice.clone())
        .near_balance()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .total;
    assert!(
        alice_balance_after.as_yoctonear()
            >= alice_balance_before.as_yoctonear() + NearToken::from_near(1).as_yoctonear(),
        "alice balance should have increased by 1 NEAR"
    );

    Ok(())
}

#[tokio::test]
async fn test_proposal_set_staking_contract() -> testresult::TestResult {
    let (ctx, dao) = setup_dao().await?;

    let test_token = setup_test_token(&ctx).await?;
    let staking = setup_staking(&ctx, &test_token.0, &dao.0).await?;

    set_staking_contract(&ctx, &dao, &staking.0).await?;

    let staking_contract: String = dao
        .call_function("get_staking_contract", json!({}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(staking_contract, staking.0.to_string());

    // Trying to set staking contract again should fail
    let result = add_proposal(
        &ctx,
        &dao,
        ProposalInput {
            description: "set staking again".to_string(),
            kind: ProposalKind::SetStakingContract {
                staking_id: staking.0.clone(),
            },
        },
    )
    .await;
    assert!(
        format!("{:?}", result.failures()).contains("ERR_STAKING_CONTRACT_CANT_CHANGE"),
        "{:?}",
        result.failures()
    );

    Ok(())
}

#[tokio::test]
async fn test_voting_only_for_councils() -> testresult::TestResult {
    let (ctx, dao) = setup_dao().await?;
    let alice = create_named_account(&ctx, "alice", 100).await?;

    let proposal_id: u64 = add_proposal_as(
        &ctx,
        &dao,
        &alice,
        ProposalInput {
            description: "rename the dao".to_string(),
            kind: ProposalKind::ChangeConfig {
                config: Config {
                    name: "sputnikdao".to_string(),
                    purpose: "testing".to_string(),
                    metadata: Base64VecU8(vec![]),
                },
            },
        },
    )
    .await
    .json()?;

    // Alice is not in council — vote should be denied
    let result = dao
        .call_function(
            "act_proposal",
            json!({
                "id": proposal_id,
                "action": Action::VoteApprove,
                "proposal": get_proposal_kind(&ctx, &dao, proposal_id).await?
            }),
        )
        .transaction()
        .max_gas()
        .with_signer(alice.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        format!("{:?}", result.failures()).contains("ERR_PERMISSION_DENIED"),
        "{:?}",
        result.failures()
    );

    let proposal: ProposalOutput = dao
        .call_function("get_proposal", json!({"id": proposal_id}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(proposal.proposal.status, ProposalStatus::InProgress);

    // Root (council member) votes — should succeed and approve
    vote(&ctx, vec![&ctx.root], &dao, proposal_id).await?;

    let proposal: ProposalOutput = dao
        .call_function("get_proposal", json!({"id": proposal_id}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(proposal.proposal.status, ProposalStatus::Approved);

    Ok(())
}

#[tokio::test]
async fn test_act_proposal_correct_kind() -> testresult::TestResult {
    let (ctx, dao) = setup_dao().await?;
    let alice = create_named_account(&ctx, "alice", 100).await?;

    let correct_config = Config {
        name: "sputnikdao".to_string(),
        purpose: "testing".to_string(),
        metadata: Base64VecU8(vec![]),
    };
    let wrong_config = Config {
        name: "sputnikdao_fake".to_string(),
        purpose: "testing".to_string(),
        metadata: Base64VecU8(vec![]),
    };

    let proposal_id: u64 = add_proposal_as(
        &ctx,
        &dao,
        &alice,
        ProposalInput {
            description: "rename the dao".to_string(),
            kind: ProposalKind::ChangeConfig {
                config: correct_config,
            },
        },
    )
    .await
    .json()?;

    // Passing a different kind should be rejected with ERR_WRONG_KIND
    let result = dao
        .call_function(
            "act_proposal",
            json!({
                "id": proposal_id,
                "action": Action::VoteApprove,
                "proposal": { "ChangeConfig": { "config": wrong_config } }
            }),
        )
        .transaction()
        .max_gas()
        .with_signer(ctx.root.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        format!("{:?}", result.failures()).contains("ERR_WRONG_KIND"),
        "{:?}",
        result.failures()
    );

    // Proposal must still be InProgress
    let proposal: ProposalOutput = dao
        .call_function("get_proposal", json!({"id": proposal_id}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(proposal.proposal.status, ProposalStatus::InProgress);

    Ok(())
}

#[tokio::test]
async fn test_proposal_group_changed_during_voting() -> testresult::TestResult {
    let (ctx, dao) = setup_dao().await?;
    let alice = create_named_account(&ctx, "alice", 100).await?;

    // Add a Transfer proposal (from root, so root is both proposer and voter)
    let transfer_id: u64 = add_transfer_proposal(
        &ctx,
        &dao,
        base_token(),
        alice.clone(),
        NearToken::from_near(1).as_yoctonear(),
        None,
    )
    .await
    .json()?;

    // Also add a proposal to add alice to council
    let add_member_id: u64 = add_member_proposal(&ctx, &dao, alice.clone())
        .await
        .json()?;

    // Approve adding alice to council first — now council has 2 members
    vote(&ctx, vec![&ctx.root], &dao, add_member_id).await?;

    // Now try to approve the original transfer — with 2 members, root's single
    // vote is no longer enough (requires majority), so it should stay InProgress
    vote(&ctx, vec![&ctx.root], &dao, transfer_id).await?;

    let proposal: ProposalOutput = dao
        .call_function("get_proposal", json!({"id": transfer_id}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(proposal.proposal.status, ProposalStatus::InProgress);

    Ok(())
}

#[tokio::test]
async fn test_proposal_transfer_ft() -> testresult::TestResult {
    let (ctx, dao) = setup_dao().await?;
    let alice = create_named_account(&ctx, "alice", 100).await?;

    let test_token = setup_test_token(&ctx).await?;

    // Mint tokens to the DAO
    test_token
        .call_function(
            "mint",
            json!({"account_id": dao.0.clone(), "amount": U128(1_000_000_000)}),
        )
        .transaction()
        .with_signer(dao.0.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .into_result()?;

    // Register alice for storage on the token contract
    near_api::StorageDeposit::on_contract(test_token.0.clone())
        .deposit(alice.clone(), NearToken::from_near(90))
        .with_signer(alice.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .into_result()?;

    let transfer_id: u64 = add_transfer_proposal(
        &ctx,
        &dao,
        Some(test_token.0.clone()),
        alice.clone(),
        10,
        None,
    )
    .await
    .json()?;

    vote(&ctx, vec![&ctx.root], &dao, transfer_id).await?;

    let proposal: ProposalOutput = dao
        .call_function("get_proposal", json!({"id": transfer_id}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(proposal.proposal.status, ProposalStatus::Approved);

    Ok(())
}

#[tokio::test]
async fn test_callback_transfer() -> testresult::TestResult {
    let (ctx, dao) = setup_dao().await?;
    let alice = create_named_account(&ctx, "alice", 100).await?;
    let user1 = create_named_account(&ctx, "user1", 100).await?;

    // Submit a broken transfer (never voted on – just to exercise add_proposal path)
    let _transfer_id: u64 = add_transfer_proposal(
        &ctx,
        &dao,
        base_token(),
        "broken.account.id".parse()?,
        NearToken::from_near(1).as_yoctonear(),
        None,
    )
    .await
    .json()?;

    // user1 proposes a broken transfer
    // (rebuild the proposal so user1 is the proposer)
    let transfer_id2: u64 = dao
        .call_function(
            "add_proposal",
            json!({
                "proposal": {
                    "description": "give me tokens",
                    "kind": {
                        "Transfer": {
                            "token_id": "",
                            "receiver_id": "broken.id",
                            "amount": U128(NearToken::from_near(1).as_yoctonear())
                        }
                    }
                }
            }),
        )
        .transaction()
        .deposit(NearToken::from_near(1))
        .with_signer(user1.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .json()?;

    let user1_balance_before = near_api::Tokens::account(user1.clone())
        .near_balance()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .total;

    vote(&ctx, vec![&ctx.root], &dao, transfer_id2).await?;

    let proposal: ProposalOutput = dao
        .call_function("get_proposal", json!({"id": transfer_id2}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(
        proposal.proposal.status,
        ProposalStatus::Failed,
        "transfer to broken id should fail"
    );
    // Bond not returned on failed transfer
    let user1_balance_after = near_api::Tokens::account(user1.clone())
        .near_balance()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .total;
    assert_eq!(
        user1_balance_before, user1_balance_after,
        "bond should not be returned on failure"
    );

    // Transfer to a real account → should succeed and return bond
    let transfer_id3: u64 = dao
        .call_function(
            "add_proposal",
            json!({
                "proposal": {
                    "description": "give me tokens",
                    "kind": {
                        "Transfer": {
                            "token_id": "",
                            "receiver_id": alice,
                            "amount": U128(NearToken::from_near(1).as_yoctonear())
                        }
                    }
                }
            }),
        )
        .transaction()
        .deposit(NearToken::from_near(1))
        .with_signer(user1.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .json()?;

    let user1_balance_before2 = near_api::Tokens::account(user1.clone())
        .near_balance()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .total;

    vote(&ctx, vec![&ctx.root], &dao, transfer_id3).await?;

    let proposal: ProposalOutput = dao
        .call_function("get_proposal", json!({"id": transfer_id3}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(proposal.proposal.status, ProposalStatus::Approved);

    // Bond returned on successful transfer
    let user1_balance_after2 = near_api::Tokens::account(user1.clone())
        .near_balance()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .total;
    assert!(
        user1_balance_after2 > user1_balance_before2,
        "bond should be returned on success: before={} after={}",
        user1_balance_before2,
        user1_balance_after2
    );

    Ok(())
}

#[tokio::test]
async fn test_callback_function_call() -> testresult::TestResult {
    let (ctx, dao) = setup_dao().await?;
    let alice = create_named_account(&ctx, "alice", 100).await?;

    let test_token = setup_test_token(&ctx).await?;

    // FunctionCall to a non-existent method ("fail") → proposal should fail
    // (No pre-registration needed: test_token.mint() calls internal_register_account)
    let transfer_id: u64 = dao
        .call_function(
            "add_proposal",
            json!({
                "proposal": {
                    "description": "call fail method",
                    "kind": {
                        "FunctionCall": {
                            "receiver_id": test_token.0.to_string(),
                            "actions": [{
                                "method_name": "fail",
                                "args": general_purpose::STANDARD.encode("bad args"),
                                "deposit": "0",
                                "gas": 10_000_000_000_000u64
                            }]
                        }
                    }
                }
            }),
        )
        .transaction()
        .deposit(NearToken::from_near(1))
        .with_signer(dao.0.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .json()?;

    dao.call_function(
        "act_proposal",
        json!({
            "id": transfer_id,
            "action": Action::VoteApprove,
            "proposal": get_proposal_kind(&ctx, &dao, transfer_id).await?
        }),
    )
    .transaction()
    .max_gas()
    .with_signer(ctx.root.clone(), ctx.signer.clone())
    .send_to(&ctx.sandbox_network)
    .await?
    .into_result()?;

    let proposal: ProposalOutput = dao
        .call_function("get_proposal", json!({"id": transfer_id}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(proposal.proposal.status, ProposalStatus::Failed);

    // Successful FunctionCall: mint then burn tokens for alice
    let args_mint =
        general_purpose::STANDARD.encode(format!(r#"{{"account_id": "{alice}", "amount": "10"}}"#));
    let args_burn =
        general_purpose::STANDARD.encode(format!(r#"{{"account_id": "{alice}", "amount": "10"}}"#));

    let transfer_id2: u64 = dao
        .call_function(
            "add_proposal",
            json!({
                "proposal": {
                    "description": "call mint+burn",
                    "kind": {
                        "FunctionCall": {
                            "receiver_id": test_token.0.to_string(),
                            "actions": [
                                {
                                    "method_name": "mint",
                                    "args": args_mint,
                                    "deposit": "0",
                                    "gas": 10_000_000_000_000u64
                                },
                                {
                                    "method_name": "burn",
                                    "args": args_burn,
                                    "deposit": "0",
                                    "gas": 10_000_000_000_000u64
                                }
                            ]
                        }
                    }
                }
            }),
        )
        .transaction()
        .deposit(NearToken::from_near(1))
        .with_signer(dao.0.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .json()?;

    dao.call_function(
        "act_proposal",
        json!({
            "id": transfer_id2,
            "action": Action::VoteApprove,
            "proposal": get_proposal_kind(&ctx, &dao, transfer_id2).await?
        }),
    )
    .transaction()
    .max_gas()
    .with_signer(ctx.root.clone(), ctx.signer.clone())
    .send_to(&ctx.sandbox_network)
    .await?
    .into_result()?;

    let proposal2: ProposalOutput = dao
        .call_function("get_proposal", json!({"id": transfer_id2}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(proposal2.proposal.status, ProposalStatus::Approved);

    Ok(())
}

#[tokio::test]
async fn test_remove_blob() -> testresult::TestResult {
    let (ctx, dao) = setup_dao().await?;
    let alice = create_named_account(&ctx, "alice", 100).await?;

    // Store the blob
    let hash: near_sdk::json_types::Base58CryptoHash = dao
        .call_function_raw("store_blob", dao_wasm_bytes().to_vec())
        .transaction()
        .deposit(NearToken::from_near(200))
        .max_gas()
        .with_signer(ctx.root.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .json()?;

    // Wrong hash → ERR_NO_BLOB
    let result = dao
        .call_function(
            "remove_blob",
            json!({"hash": "HLBiX51txizmQzZJMrHMCq4u7iEEqNbaJppZ84yW7628"}),
        )
        .transaction()
        .with_signer(ctx.root.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        format!("{:?}", result.failures()).contains("ERR_NO_BLOB"),
        "{:?}",
        result.failures()
    );

    // Wrong caller (alice) → ERR_INVALID_CALLER
    let result = dao
        .call_function("remove_blob", json!({"hash": hash}))
        .transaction()
        .with_signer(alice.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        format!("{:?}", result.failures()).contains("ERR_INVALID_CALLER"),
        "{:?}",
        result.failures()
    );

    // Correct removal — root's balance should increase (refund for storage)
    let root_balance_before = near_api::Tokens::account(ctx.root.clone())
        .near_balance()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .total;

    dao.call_function("remove_blob", json!({"hash": hash}))
        .transaction()
        .with_signer(ctx.root.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .into_result()?;

    let root_balance_after = near_api::Tokens::account(ctx.root.clone())
        .near_balance()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .total;

    let has_blob: bool = dao
        .call_function("has_blob", json!({"hash": hash}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert!(!has_blob);
    assert!(
        root_balance_after > root_balance_before,
        "root should have received storage refund: before={} after={}",
        root_balance_before,
        root_balance_after
    );

    Ok(())
}
