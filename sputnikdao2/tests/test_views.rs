use near_sdk::json_types::{Base64VecU8, U64, U128};
use near_sdk::serde_json::json;

use near_api::NearToken;
use sputnikdao2::{
    BountyOutput, Config, Policy, ProposalInput, ProposalKind, ProposalOutput, ProposalStatus,
    RoleKind, VotePolicy,
};

mod utils;
use crate::utils::*;

#[tokio::test]
async fn test_view_version() -> testresult::TestResult {
    let (ctx, dao) = setup_dao().await?;

    let version: String = dao
        .call_function("version", json!({}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;

    assert_eq!(version, env!("CARGO_PKG_VERSION"));

    Ok(())
}

#[tokio::test]
async fn test_view_get_config() -> testresult::TestResult {
    let (ctx, dao) = setup_dao().await?;

    // The setup_dao uses Config { name: "test", purpose: "to test", metadata: [] }
    let config: Config = dao
        .call_function("get_config", json!({}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;

    assert_eq!(config.name, "test");
    assert_eq!(config.purpose, "to test");
    assert_eq!(config.metadata, Base64VecU8(vec![]));

    Ok(())
}

#[tokio::test]
async fn test_view_get_policy() -> testresult::TestResult {
    let (ctx, dao) = setup_dao().await?;

    let policy: Policy = dao
        .call_function("get_policy", json!({}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;

    // Default policy has two roles: "all" (Everyone) and "council" (Group(root))
    assert_eq!(policy.roles.len(), 2);

    let all_role = policy.roles.iter().find(|r| r.name == "all").unwrap();
    assert_eq!(all_role.kind, RoleKind::Everyone);
    assert!(all_role.permissions.contains("*:AddProposal"));

    let council_role = policy.roles.iter().find(|r| r.name == "council").unwrap();
    assert_eq!(
        council_role.kind,
        RoleKind::Group(vec![ctx.root.clone()].into_iter().collect())
    );

    // Default vote policy: RoleWeight, quorum=0, threshold=1/2
    assert_eq!(policy.default_vote_policy, VotePolicy::default());

    assert_eq!(policy.proposal_bond, NearToken::from_near(1));
    assert_eq!(
        policy.proposal_period,
        U64::from(1_000_000_000u64 * 60 * 60 * 24 * 7)
    );
    assert_eq!(policy.bounty_bond, NearToken::from_near(1));
    assert_eq!(
        policy.bounty_forgiveness_period,
        U64::from(1_000_000_000u64 * 60 * 60 * 24)
    );

    Ok(())
}

#[tokio::test]
async fn test_view_get_staking_contract() -> testresult::TestResult {
    let (ctx, dao) = setup_dao().await?;

    let staking_contract: String = dao
        .call_function("get_staking_contract", json!({}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert!(staking_contract.is_empty());

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

    Ok(())
}

#[tokio::test]
async fn test_view_has_blob() -> testresult::TestResult {
    let (ctx, dao) = setup_dao().await?;

    let hash: near_sdk::json_types::Base58CryptoHash = dao
        .call_function_raw("store_blob", dao_wasm_bytes().to_vec())
        .transaction()
        .deposit(NearToken::from_near(200))
        .max_gas()
        .with_signer(ctx.root.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .json()?;

    let has: bool = dao
        .call_function("has_blob", json!({"hash": hash}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert!(has);

    dao.call_function("remove_blob", json!({"hash": hash}))
        .transaction()
        .with_signer(ctx.root.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .into_result()?;

    let has: bool = dao
        .call_function("has_blob", json!({"hash": hash}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert!(!has);

    Ok(())
}

#[tokio::test]
async fn test_view_get_locked_storage_amount() -> testresult::TestResult {
    let (ctx, dao) = setup_dao().await?;

    let before: NearToken = dao
        .call_function("get_locked_storage_amount", json!({}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;

    add_proposal(
        &ctx,
        &dao,
        ProposalInput {
            description: "adding some bytes".to_string(),
            kind: ProposalKind::Vote,
        },
    )
    .await
    .into_result()?;

    let after: NearToken = dao
        .call_function("get_locked_storage_amount", json!({}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;

    assert!(
        after > before,
        "locked storage should increase after adding a proposal: before={} after={}",
        before,
        after
    );

    Ok(())
}

#[tokio::test]
async fn test_view_get_available_amount() -> testresult::TestResult {
    let (ctx, dao) = setup_dao().await?;

    let before: NearToken = dao
        .call_function("get_available_amount", json!({}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;

    add_proposal(
        &ctx,
        &dao,
        ProposalInput {
            description: "adding some bytes".to_string(),
            kind: ProposalKind::Vote,
        },
    )
    .await
    .into_result()?;

    let after: NearToken = dao
        .call_function("get_available_amount", json!({}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;

    assert!(
        before > after,
        "available amount should decrease after adding a proposal: before={} after={}",
        before,
        after
    );

    Ok(())
}

#[tokio::test]
async fn test_view_delegation_methods() -> testresult::TestResult {
    let (ctx, dao) = setup_dao().await?;
    let alice = create_named_account(&ctx, "alice", 100).await?;
    let bob = create_named_account(&ctx, "bob", 100).await?;

    let test_token = setup_test_token(&ctx).await?;
    let staking = setup_staking(&ctx, &test_token.0, &dao.0).await?;

    set_staking_contract(&ctx, &dao, &staking.0).await?;

    let random_amount: u128 = 10_087_687_667_869;

    register_and_delegate(&ctx, &dao, &staking, &alice, random_amount).await?;
    register_and_delegate(&ctx, &dao, &staking, &bob, random_amount * 2).await?;

    let alice_bal: U128 = dao
        .call_function("delegation_balance_of", json!({"account_id": alice}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(alice_bal, U128(random_amount));

    let bob_bal: U128 = dao
        .call_function("delegation_balance_of", json!({"account_id": bob}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(bob_bal, U128(random_amount * 2));

    let total: U128 = dao
        .call_function("delegation_total_supply", json!({}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(total, U128(random_amount * 3));

    let ratio: (U128, U128) = dao
        .call_function("delegation_balance_ratio", json!({"account_id": alice}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(ratio.0, alice_bal);
    assert_eq!(ratio.1, total);

    Ok(())
}

#[tokio::test]
async fn test_view_proposal_methods() -> testresult::TestResult {
    let (ctx, dao) = setup_dao().await?;
    let alice = create_named_account(&ctx, "alice", 100).await?;

    let last_id: u64 = dao
        .call_function("get_last_proposal_id", json!({}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(last_id, 0);

    let proposals: Vec<ProposalOutput> = dao
        .call_function("get_proposals", json!({"from_index": 0, "limit": 100}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert!(proposals.is_empty());

    let config = Config {
        name: "sputnikdao2".to_string(),
        purpose: "testing_view_methods".to_string(),
        metadata: Base64VecU8(vec![]),
    };
    add_proposal_as(
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
    .into_result()?;

    let proposal: ProposalOutput = dao
        .call_function("get_proposal", json!({"id": 0}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(proposal.proposal.proposer, alice);
    assert_eq!(proposal.proposal.description, "rename the dao");
    assert_eq!(proposal.proposal.status, ProposalStatus::InProgress);
    assert!(proposal.proposal.vote_counts.is_empty());
    assert!(proposal.proposal.votes.is_empty());

    let last_id: u64 = dao
        .call_function("get_last_proposal_id", json!({}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(last_id, 1);

    let proposals: Vec<ProposalOutput> = dao
        .call_function("get_proposals", json!({"from_index": 0, "limit": 100}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(proposals.len(), 1);
    assert_eq!(proposals[0].proposal.proposer, alice);

    let result = dao
        .call_function("get_proposal", json!({"id": 10}))
        .read_only::<ProposalOutput>()
        .fetch_from(&ctx.sandbox_network)
        .await;
    assert!(result.is_err());
    assert!(
        format!("{:?}", result.unwrap_err()).contains("ERR_NO_PROPOSAL"),
        "Expected ERR_NO_PROPOSAL"
    );

    Ok(())
}

#[tokio::test]
async fn test_view_bounty_methods() -> testresult::TestResult {
    let (ctx, dao) = setup_dao().await?;
    let alice = create_named_account(&ctx, "alice", 100).await?;

    let last_bounty_id: u64 = dao
        .call_function("get_last_bounty_id", json!({}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(last_bounty_id, 0);

    let bounties: Vec<BountyOutput> = dao
        .call_function("get_bounties", json!({"from_index": 0, "limit": 100}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert!(bounties.is_empty());

    let test_token = setup_test_token(&ctx).await?;
    let proposal_id = propose_bounty_ft(&ctx, &dao, &alice, &test_token.0).await?;

    vote_approve(&ctx, &ctx.root, &dao, proposal_id).await?;

    let last_bounty_id: u64 = dao
        .call_function("get_last_bounty_id", json!({}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(last_bounty_id, 1);

    let bounties: Vec<BountyOutput> = dao
        .call_function("get_bounties", json!({"from_index": 0, "limit": 100}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(bounties.len(), 1);
    assert_eq!(bounties[0].bounty.token, test_token.0.to_string());
    assert_eq!(bounties[0].bounty.times, 3);
    assert_eq!(bounties[0].bounty.max_deadline, U64(DEADLINE));

    let bounty: BountyOutput = dao
        .call_function("get_bounty", json!({"id": 0}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(bounty.bounty.description, "test_bounties");
    assert_eq!(bounty.bounty.times, 3);

    claim_bounty_as(&ctx, &dao, &alice, proposal_id).await?;

    let claims_count: u32 = dao
        .call_function("get_bounty_number_of_claims", json!({"id": 0}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(claims_count, 1);

    let claims: Vec<near_sdk::serde_json::Value> = dao
        .call_function("get_bounty_claims", json!({"account_id": alice}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(claims.len(), 1);
    assert_eq!(claims[0]["bounty_id"], 0);
    assert_eq!(claims[0]["deadline"], DEADLINE.to_string());
    assert_eq!(claims[0]["completed"], false);

    let result = dao
        .call_function("get_bounty", json!({"id": 10}))
        .read_only::<BountyOutput>()
        .fetch_from(&ctx.sandbox_network)
        .await;
    assert!(result.is_err());
    assert!(
        format!("{:?}", result.err().unwrap()).contains("ERR_NO_BOUNTY"),
        "Expected ERR_NO_BOUNTY"
    );

    Ok(())
}
