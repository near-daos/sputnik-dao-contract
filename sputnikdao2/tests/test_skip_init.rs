use near_api::NearToken;
use near_sdk::serde_json::{Value, json};
use sputnikdao2::{Action, Config, ProposalInput, ProposalKind, ProposalOutput, ProposalStatus};

mod utils;
use crate::utils::*;

#[tokio::test]
async fn test_upgrade_self_negative() -> testresult::TestResult {
    let (ctx, dao) = deploy_dao_no_init().await?;

    // NOT INITIALIZED: store_blob must fail with ERR_CONTRACT_IS_NOT_INITIALIZED
    let result = dao
        .call_function_raw("store_blob", dao_wasm_bytes().to_vec())
        .transaction()
        .deposit(NearToken::from_near(200))
        .max_gas()
        .with_signer(ctx.root.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        format!("{:?}", result.failures()).contains("ERR_CONTRACT_IS_NOT_INITIALIZED"),
        "Expected ERR_CONTRACT_IS_NOT_INITIALIZED, got: {:?}",
        result.failures()
    );

    // Initialize the contract
    let config = json!({"name": "sputnik", "purpose": "testing", "metadata": ""});
    dao.call_function(
        "new",
        json!({
            "config": config,
            "policy": [ctx.root.to_string()]
        }),
    )
    .transaction()
    .with_signer(ctx.root.clone(), ctx.signer.clone())
    .send_to(&ctx.sandbox_network)
    .await?
    .into_result()?;

    // Too small deposit → ERR_NOT_ENOUGH_DEPOSIT
    let result = dao
        .call_function_raw("store_blob", dao_wasm_bytes().to_vec())
        .transaction()
        .deposit(NearToken::from_near(1))
        .max_gas()
        .with_signer(ctx.root.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        format!("{:?}", result.failures()).contains("ERR_NOT_ENOUGH_DEPOSIT"),
        "Expected ERR_NOT_ENOUGH_DEPOSIT, got: {:?}",
        result.failures()
    );

    // Store successfully with enough deposit
    dao.call_function_raw("store_blob", dao_wasm_bytes().to_vec())
        .transaction()
        .deposit(NearToken::from_near(200))
        .max_gas()
        .with_signer(ctx.root.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .into_result()?;

    // Same blob again → ERR_ALREADY_EXISTS
    let result = dao
        .call_function_raw("store_blob", dao_wasm_bytes().to_vec())
        .transaction()
        .deposit(NearToken::from_near(200))
        .max_gas()
        .with_signer(ctx.root.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        format!("{:?}", result.failures()).contains("ERR_ALREADY_EXISTS"),
        "Expected ERR_ALREADY_EXISTS, got: {:?}",
        result.failures()
    );

    Ok(())
}

#[tokio::test]
async fn test_proposal_action_types() -> testresult::TestResult {
    let (ctx, dao) = deploy_dao_no_init().await?;
    let alice = create_named_account(&ctx, "alice", 100).await?;
    let user1 = create_named_account(&ctx, "user1", 100).await?;
    let user2 = create_named_account(&ctx, "user2", 100).await?;
    let user3 = create_named_account(&ctx, "user3", 100).await?;

    let period = (1_000_000_000u64 * 60 * 60 * 24 * 7).to_string();
    let yocto_1 = NearToken::from_near(1).as_yoctonear().to_string();

    // Initialize DAO with alice, user1, user2, user3 in council (*:*)
    dao.call_function(
        "new",
        json!({
            "config": {"name": "sputnik", "purpose": "testing", "metadata": ""},
            "policy": {
                "roles": [{
                    "name": "council",
                    "kind": {"Group": [
                        alice.to_string(), user1.to_string(),
                        user2.to_string(), user3.to_string()
                    ]},
                    "permissions": ["*:*"],
                    "vote_policy": {}
                }],
                "default_vote_policy": {
                    "weight_kind": "RoleWeight",
                    "quorum": "0",
                    "threshold": [1, 2]
                },
                "proposal_bond": yocto_1,
                "proposal_period": period,
                "bounty_bond": yocto_1,
                "bounty_forgiveness_period": period
            }
        }),
    )
    .transaction()
    .with_signer(ctx.root.clone(), ctx.signer.clone())
    .send_to(&ctx.sandbox_network)
    .await?
    .into_result()?;

    let config = Config {
        name: "sputnik".to_string(),
        purpose: "testing".to_string(),
        metadata: near_sdk::json_types::Base64VecU8(vec![]),
    };

    // Add a ChangeConfig proposal as alice
    let proposal_id: u64 = add_proposal_as(
        &ctx,
        &dao,
        &alice,
        ProposalInput {
            description: "rename the dao".to_string(),
            kind: ProposalKind::ChangeConfig {
                config: config.clone(),
            },
        },
    )
    .await
    .json()?;

    // RemoveProposal works
    dao.call_function(
        "act_proposal",
        json!({
            "id": proposal_id,
            "action": Action::RemoveProposal,
            "proposal": get_proposal_kind(&ctx, &dao, proposal_id).await?
        }),
    )
    .transaction()
    .with_signer(alice.clone(), ctx.signer.clone())
    .send_to(&ctx.sandbox_network)
    .await?
    .into_result()?;

    // Proposal is gone → ERR_NO_PROPOSAL on view
    let result = dao
        .call_function("get_proposal", json!({"id": proposal_id}))
        .read_only::<ProposalOutput>()
        .fetch_from(&ctx.sandbox_network)
        .await;
    assert!(
        format!("{:?}", result.unwrap_err()).contains("ERR_NO_PROPOSAL"),
        "Expected ERR_NO_PROPOSAL after RemoveProposal"
    );

    // VoteApprove on removed proposal → ERR_NO_PROPOSAL
    // Pass a valid ProposalKind (Vote unit variant) so deserialization succeeds;
    // the contract will panic at the storage lookup before checking the kind.
    let result = dao
        .call_function(
            "act_proposal",
            json!({"id": proposal_id, "action": Action::VoteApprove, "proposal": "Vote"}),
        )
        .transaction()
        .with_signer(alice.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        format!("{:?}", result.failures()).contains("ERR_NO_PROPOSAL"),
        "Expected ERR_NO_PROPOSAL on VoteApprove of removed proposal: {:?}",
        result.failures()
    );

    // Add a second proposal
    let proposal_id: u64 = add_proposal_as(
        &ctx,
        &dao,
        &alice,
        ProposalInput {
            description: "rename the dao".to_string(),
            kind: ProposalKind::ChangeConfig {
                config: config.clone(),
            },
        },
    )
    .await
    .json()?;

    // AddProposal as action → ERR_WRONG_ACTION
    let result = dao
        .call_function(
            "act_proposal",
            json!({
                "id": proposal_id,
                "action": Action::AddProposal,
                "proposal": get_proposal_kind(&ctx, &dao, proposal_id).await?
            }),
        )
        .transaction()
        .with_signer(alice.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        format!("{:?}", result.failures()).contains("ERR_WRONG_ACTION"),
        "Expected ERR_WRONG_ACTION: {:?}",
        result.failures()
    );

    // Cast 3 different votes; capture the kind once and reuse
    let proposal_kind = get_proposal_kind(&ctx, &dao, proposal_id).await?;

    dao.call_function(
        "act_proposal",
        json!({
            "id": proposal_id,
            "action": Action::VoteApprove,
            "proposal": proposal_kind.clone()
        }),
    )
    .transaction()
    .with_signer(user1.clone(), ctx.signer.clone())
    .send_to(&ctx.sandbox_network)
    .await?
    .into_result()?;

    dao.call_function(
        "act_proposal",
        json!({
            "id": proposal_id,
            "action": Action::VoteReject,
            "proposal": proposal_kind.clone()
        }),
    )
    .transaction()
    .with_signer(user2.clone(), ctx.signer.clone())
    .send_to(&ctx.sandbox_network)
    .await?
    .into_result()?;

    dao.call_function(
        "act_proposal",
        json!({
            "id": proposal_id,
            "action": Action::VoteRemove,
            "proposal": proposal_kind.clone()
        }),
    )
    .transaction()
    .with_signer(alice.clone(), ctx.signer.clone())
    .send_to(&ctx.sandbox_network)
    .await?
    .into_result()?;

    // Check vote_counts and votes.
    // ProposalOutput has #[serde(flatten)] so all Proposal fields are at the top level.
    let proposal_val: Value = dao
        .call_function("get_proposal", json!({"id": proposal_id}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;

    assert_eq!(
        proposal_val["vote_counts"]["council"],
        json!(["1", "1", "1"]),
        "vote_counts should be [approve=1, reject=1, remove=1]"
    );
    assert_eq!(proposal_val["votes"][alice.to_string()], json!("Remove"));
    assert_eq!(proposal_val["votes"][user1.to_string()], json!("Approve"));
    assert_eq!(proposal_val["votes"][user2.to_string()], json!("Reject"));

    // Finalize a non-expired, non-failed proposal → ERR_PROPOSAL_NOT_EXPIRED_OR_FAILED
    let result = dao
        .call_function(
            "act_proposal",
            json!({
                "id": proposal_id,
                "action": Action::Finalize,
                "proposal": proposal_kind
            }),
        )
        .transaction()
        .with_signer(alice.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        format!("{:?}", result.failures()).contains("ERR_PROPOSAL_NOT_EXPIRED_OR_FAILED"),
        "Expected ERR_PROPOSAL_NOT_EXPIRED_OR_FAILED: {:?}",
        result.failures()
    );

    Ok(())
}

#[tokio::test]
async fn test_policy_token_weight() -> testresult::TestResult {
    let (ctx, dao) = deploy_dao_no_init().await?;
    let alice = create_named_account(&ctx, "alice", 100).await?;
    let bob = create_named_account(&ctx, "bob", 100).await?;

    // Set up staking infrastructure before initializing the DAO
    let test_token = setup_test_token(&ctx).await?;
    let staking = setup_staking(&ctx, &test_token.0, &dao.0).await?;

    // Initialize the DAO with default policy (root only)
    dao.call_function(
        "new",
        json!({
            "config": {"name": "sputnik", "purpose": "testing", "metadata": ""},
            "policy": [ctx.root.to_string()]
        }),
    )
    .transaction()
    .with_signer(ctx.root.clone(), ctx.signer.clone())
    .send_to(&ctx.sandbox_network)
    .await?
    .into_result()?;

    // Set staking contract
    set_staking_contract(&ctx, &dao, &staking.0).await?;

    let period = (1_000_000_000u64 * 60 * 60 * 24 * 7).to_string();
    let yocto_1 = NearToken::from_near(1).as_yoctonear().to_string();

    // Propose changing to a TokenWeight policy with alice+bob in group, threshold=5
    let token_weight_policy = json!({
        "roles": [{
            "name": "all",
            "kind": {"Group": [alice.to_string(), bob.to_string()]},
            "permissions": ["*:AddProposal", "*:VoteApprove"],
            "vote_policy": {}
        }],
        "default_vote_policy": {
            "weight_kind": "TokenWeight",
            "quorum": "1",
            "threshold": "5"
        },
        "proposal_bond": yocto_1,
        "proposal_period": period,
        "bounty_bond": yocto_1,
        "bounty_forgiveness_period": period
    });

    let change_policy_id: u64 = dao
        .call_function(
            "add_proposal",
            json!({
                "proposal": {
                    "description": "test",
                    "kind": {"ChangePolicy": {"policy": token_weight_policy}}
                }
            }),
        )
        .transaction()
        .deposit(NearToken::from_near(1))
        .with_signer(ctx.root.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .json()?;

    // Root votes approve → policy changes
    vote_approve(&ctx, &ctx.root, &dao, change_policy_id).await?;

    // Register and delegate: alice=1, bob=4 (total=5)
    register_and_delegate(&ctx, &dao, &staking, &alice, 1).await?;
    register_and_delegate(&ctx, &dao, &staking, &bob, 4).await?;

    // Alice proposes a config change
    let new_config = json!({"name": "new dao wohoo", "purpose": "testing", "metadata": ""});
    let proposal_id: u64 = dao
        .call_function(
            "add_proposal",
            json!({
                "proposal": {
                    "description": "test",
                    "kind": {"ChangeConfig": {"config": new_config}}
                }
            }),
        )
        .transaction()
        .deposit(NearToken::from_near(1))
        .with_signer(alice.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .json()?;

    // alice (1 token) + bob (4 tokens) = 5 ≥ threshold(5) → approved
    vote_approve(&ctx, &alice, &dao, proposal_id).await?;
    vote_approve(&ctx, &bob, &dao, proposal_id).await?;

    // Config should now be updated
    let config: Value = dao
        .call_function("get_config", json!({}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(config["name"], json!("new dao wohoo"));
    assert_eq!(config["purpose"], json!("testing"));

    Ok(())
}

#[tokio::test]
async fn test_policy_self_lock() -> testresult::TestResult {
    let (ctx, dao) = deploy_dao_no_init().await?;
    let alice = create_named_account(&ctx, "alice", 100).await?;

    let period = (1_000_000_000u64 * 60 * 60 * 24 * 7).to_string();
    let yocto_1 = NearToken::from_near(1).as_yoctonear().to_string();

    // TokenWeight policy with only alice; staking_id never set → no one can delegate
    let policy = json!({
        "roles": [{
            "name": "all",
            "kind": {"Group": [alice.to_string()]},
            "permissions": ["*:AddProposal", "*:VoteApprove"],
            "vote_policy": {}
        }],
        "default_vote_policy": {
            "weight_kind": "TokenWeight",
            "quorum": "1",
            "threshold": "5"
        },
        "proposal_bond": yocto_1,
        "proposal_period": period,
        "bounty_bond": yocto_1,
        "bounty_forgiveness_period": period
    });

    dao.call_function(
        "new",
        json!({
            "config": {"name": "sputnik", "purpose": "testing", "metadata": ""},
            "policy": policy.clone()
        }),
    )
    .transaction()
    .with_signer(ctx.root.clone(), ctx.signer.clone())
    .send_to(&ctx.sandbox_network)
    .await?
    .into_result()?;

    // Alice adds a ChangePolicy proposal
    let proposal_id: u64 = dao
        .call_function(
            "add_proposal",
            json!({
                "proposal": {
                    "description": "test",
                    "kind": {"ChangePolicy": {"policy": policy.clone()}}
                }
            }),
        )
        .transaction()
        .deposit(NearToken::from_near(1))
        .with_signer(alice.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .json()?;

    // Alice votes approve – but her token weight is 0 (no staking set up)
    // The vote is recorded but quorum/threshold can't be met
    dao.call_function(
        "act_proposal",
        json!({
            "id": proposal_id,
            "action": Action::VoteApprove,
            "proposal": {"ChangePolicy": {"policy": policy}}
        }),
    )
    .transaction()
    .with_signer(alice.clone(), ctx.signer.clone())
    .send_to(&ctx.sandbox_network)
    .await?
    .into_result()?;

    // Proposal must remain InProgress (contract is locked)
    let proposal: ProposalOutput = dao
        .call_function("get_proposal", json!({"id": proposal_id}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(proposal.proposal.status, ProposalStatus::InProgress);

    Ok(())
}
