use near_sdk::json_types::{U128, U64};
use near_sdk::serde_json::{Value, json};

use near_api::NearToken;
use sputnikdao2::{ProposalOutput, ProposalStatus};

mod utils;
use crate::utils::*;

// ---------------------------------------------------------------------------
// Bounty tests ported from bounties.ava.ts and lib.ava.ts
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_bounty_claim_errors() -> testresult::TestResult {
    let (ctx, dao) = setup_dao().await?;
    let alice = create_named_account(&ctx, "alice", 100).await?;

    let test_token = setup_test_token(&ctx).await?;
    let proposal_id = propose_bounty_ft(&ctx, &dao, &alice, &test_token.0).await?;

    // Cannot claim before proposal is approved (bounty doesn't exist yet)
    let result = dao
        .call_function(
            "bounty_claim",
            json!({"id": proposal_id, "deadline": U64(DEADLINE)}),
        )
        .transaction()
        .deposit(NearToken::from_near(1))
        .with_signer(alice.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        format!("{:?}", result.failures()).contains("ERR_NO_BOUNTY"),
        "{:?}",
        result.failures()
    );

    // Vote to approve the bounty proposal
    vote_approve(&ctx, &ctx.root, &dao, proposal_id).await?;

    // Wrong bond amount (more than needed) → ERR_BOUNTY_WRONG_BOND
    let result = dao
        .call_function(
            "bounty_claim",
            json!({"id": proposal_id, "deadline": U64(DEADLINE)}),
        )
        .transaction()
        .deposit(NearToken::from_yoctonear(
            NearToken::from_near(1).as_yoctonear() + 1,
        ))
        .with_signer(alice.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        format!("{:?}", result.failures()).contains("ERR_BOUNTY_WRONG_BOND"),
        "{:?}",
        result.failures()
    );

    // Wrong bond amount (less than needed) → ERR_BOUNTY_WRONG_BOND
    let result = dao
        .call_function(
            "bounty_claim",
            json!({"id": proposal_id, "deadline": U64(DEADLINE)}),
        )
        .transaction()
        .deposit(NearToken::from_yoctonear(
            NearToken::from_near(1).as_yoctonear() - 1,
        ))
        .with_signer(alice.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        format!("{:?}", result.failures()).contains("ERR_BOUNTY_WRONG_BOND"),
        "{:?}",
        result.failures()
    );

    // Wrong deadline (greater than max_deadline) → ERR_BOUNTY_WRONG_DEADLINE
    let result = dao
        .call_function(
            "bounty_claim",
            json!({"id": proposal_id, "deadline": U64(DEADLINE + 1)}),
        )
        .transaction()
        .deposit(NearToken::from_near(1))
        .with_signer(alice.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        format!("{:?}", result.failures()).contains("ERR_BOUNTY_WRONG_DEADLINE"),
        "{:?}",
        result.failures()
    );

    // Claim bounty 3 times (times=3)
    claim_bounty_as(&ctx, &dao, &alice, proposal_id).await?;
    assert_eq!(
        1u32,
        dao.call_function(
            "get_bounty_number_of_claims",
            json!({"id": proposal_id})
        )
        .read_only::<u32>()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data
    );

    let claims: Vec<Value> = dao
        .call_function("get_bounty_claims", json!({"account_id": alice}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(claims[0]["bounty_id"], 0);
    assert_eq!(claims[0]["deadline"], DEADLINE.to_string());
    assert_eq!(claims[0]["completed"], false);

    claim_bounty_as(&ctx, &dao, &alice, proposal_id).await?;
    assert_eq!(
        2u32,
        dao.call_function(
            "get_bounty_number_of_claims",
            json!({"id": proposal_id})
        )
        .read_only::<u32>()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data
    );

    claim_bounty_as(&ctx, &dao, &alice, proposal_id).await?;
    assert_eq!(
        3u32,
        dao.call_function(
            "get_bounty_number_of_claims",
            json!({"id": proposal_id})
        )
        .read_only::<u32>()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data
    );

    // All 3 slots claimed — next claim should fail
    let result = dao
        .call_function(
            "bounty_claim",
            json!({"id": proposal_id, "deadline": U64(DEADLINE)}),
        )
        .transaction()
        .deposit(NearToken::from_near(1))
        .with_signer(alice.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        format!("{:?}", result.failures()).contains("ERR_BOUNTY_ALL_CLAIMED"),
        "{:?}",
        result.failures()
    );

    Ok(())
}

#[tokio::test]
async fn test_bounty_done_near() -> testresult::TestResult {
    let (ctx, dao) = setup_dao().await?;
    let alice = create_named_account(&ctx, "alice", 100).await?;
    let bob = create_named_account(&ctx, "bob", 100).await?;

    let proposal_id = propose_bounty_near(&ctx, &dao, &alice).await?;
    vote_approve(&ctx, &ctx.root, &dao, proposal_id).await?;
    claim_bounty_as(&ctx, &dao, &alice, proposal_id).await?;

    // Caller not in claims list (trying to done for bob who hasn't claimed)
    let result = dao
        .call_function(
            "bounty_done",
            json!({
                "id": proposal_id,
                "account_id": bob,
                "description": "This bounty is done"
            }),
        )
        .transaction()
        .deposit(NearToken::from_near(1))
        .with_signer(alice.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        format!("{:?}", result.failures()).contains("ERR_NO_BOUNTY_CLAIMS"),
        "{:?}",
        result.failures()
    );

    // Bob claims the bounty too
    claim_bounty_as(&ctx, &dao, &bob, proposal_id).await?;

    // Wrong bounty id → ERR_NO_BOUNTY_CLAIM
    let result = dao
        .call_function(
            "bounty_done",
            json!({
                "id": proposal_id + 10,
                "account_id": alice,
                "description": "done"
            }),
        )
        .transaction()
        .deposit(NearToken::from_near(1))
        .with_signer(alice.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        format!("{:?}", result.failures()).contains("ERR_NO_BOUNTY_CLAIM"),
        "{:?}",
        result.failures()
    );

    // Claimer tries to done for someone else → ERR_BOUNTY_DONE_MUST_BE_SELF
    let result = dao
        .call_function(
            "bounty_done",
            json!({
                "id": proposal_id,
                "account_id": bob,
                "description": "done"
            }),
        )
        .transaction()
        .deposit(NearToken::from_near(1))
        .with_signer(alice.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        format!("{:?}", result.failures()).contains("ERR_BOUNTY_DONE_MUST_BE_SELF"),
        "{:?}",
        result.failures()
    );

    // Alice's claim should still be not completed
    let claims: Vec<Value> = dao
        .call_function("get_bounty_claims", json!({"account_id": alice}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(claims[0]["completed"], false);

    // Alice successfully submits bounty done for herself
    done_bounty_as(&ctx, &dao, &alice, proposal_id, &alice).await?;

    // Claim is now marked completed
    let claims: Vec<Value> = dao
        .call_function("get_bounty_claims", json!({"account_id": alice}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(claims[0]["completed"], true);

    // The bounty_done proposal is InProgress (needs voting)
    let bounty_done_proposal: ProposalOutput = dao
        .call_function("get_proposal", json!({"id": proposal_id + 1}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(
        bounty_done_proposal.proposal.status,
        ProposalStatus::InProgress
    );

    // Vote to approve the bounty done
    vote_approve(&ctx, &ctx.root, &dao, proposal_id + 1).await?;

    let bounty_done_proposal: ProposalOutput = dao
        .call_function("get_proposal", json!({"id": proposal_id + 1}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(
        bounty_done_proposal.proposal.status,
        ProposalStatus::Approved
    );

    // Trying to done a completed bounty → ERR_NO_BOUNTY_CLAIMS (claim was removed)
    let result = dao
        .call_function(
            "bounty_done",
            json!({
                "id": proposal_id,
                "account_id": alice,
                "description": "done again"
            }),
        )
        .transaction()
        .deposit(NearToken::from_near(1))
        .with_signer(alice.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        format!("{:?}", result.failures()).contains("ERR_NO_BOUNTY_CLAIMS"),
        "{:?}",
        result.failures()
    );

    Ok(())
}

#[tokio::test]
async fn test_bounty_giveup() -> testresult::TestResult {
    let (ctx, dao) = setup_dao().await?;
    let alice = create_named_account(&ctx, "alice", 100).await?;
    let bob = create_named_account(&ctx, "bob", 100).await?;

    let test_token = setup_test_token(&ctx).await?;
    let proposal_id = propose_bounty_ft(&ctx, &dao, &alice, &test_token.0).await?;
    vote_approve(&ctx, &ctx.root, &dao, proposal_id).await?;
    claim_bounty_as(&ctx, &dao, &alice, proposal_id).await?;

    // Bob hasn't claimed → ERR_NO_BOUNTY_CLAIMS
    let result = dao
        .call_function("bounty_giveup", json!({"id": proposal_id}))
        .transaction()
        .with_signer(bob.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        format!("{:?}", result.failures()).contains("ERR_NO_BOUNTY_CLAIMS"),
        "{:?}",
        result.failures()
    );

    // Wrong bounty id → ERR_NO_BOUNTY_CLAIM
    let result = dao
        .call_function("bounty_giveup", json!({"id": proposal_id + 10}))
        .transaction()
        .with_signer(alice.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        format!("{:?}", result.failures()).contains("ERR_NO_BOUNTY_CLAIM"),
        "{:?}",
        result.failures()
    );

    // Successful giveup: alice should receive bond back
    let alice_balance_before = near_api::Tokens::account(alice.clone())
        .near_balance()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .total;

    giveup_bounty_as(&ctx, &dao, &alice, proposal_id).await?;

    let alice_balance_after = near_api::Tokens::account(alice.clone())
        .near_balance()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .total;

    // Alice gets the bond back (1 NEAR) minus gas
    assert!(
        alice_balance_after > alice_balance_before,
        "alice should receive bond refund after giveup: before={} after={}",
        alice_balance_before,
        alice_balance_after
    );

    // Claim should be gone
    let claims: Vec<Value> = dao
        .call_function("get_bounty_claims", json!({"account_id": alice}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert!(
        claims.is_empty(),
        "claim list should be empty after giveup"
    );

    Ok(())
}

#[tokio::test]
async fn test_bounty_ft_done() -> testresult::TestResult {
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

    // Register alice on the token
    near_api::StorageDeposit::on_contract(test_token.0.clone())
        .deposit(alice.clone(), NearToken::from_near(90))
        .with_signer(alice.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .into_result()?;

    // Propose a small FT bounty (amount=10)
    let proposal_id: u64 = dao
        .call_function(
            "add_proposal",
            json!({
                "proposal": {
                    "description": "add_new_bounty",
                    "kind": {
                        "AddBounty": {
                            "bounty": {
                                "description": "test_bounties",
                                "token": test_token.0,
                                "amount": "10",
                                "times": 3,
                                "max_deadline": U64(DEADLINE)
                            }
                        }
                    }
                }
            }),
        )
        .transaction()
        .deposit(NearToken::from_near(1))
        .with_signer(alice.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .json()?;

    vote_approve(&ctx, &ctx.root, &dao, proposal_id).await?;

    let proposal: ProposalOutput = dao
        .call_function("get_proposal", json!({"id": proposal_id}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(proposal.proposal.status, ProposalStatus::Approved);

    let bounty_id = 0u64; // first bounty

    claim_bounty_as(&ctx, &dao, &alice, bounty_id).await?;

    done_bounty_as(&ctx, &dao, &alice, bounty_id, &alice).await?;

    vote_approve(&ctx, &ctx.root, &dao, proposal_id + 1).await?;

    let bounty_done_proposal: ProposalOutput = dao
        .call_function("get_proposal", json!({"id": proposal_id + 1}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(
        bounty_done_proposal.proposal.status,
        ProposalStatus::Approved
    );

    Ok(())
}

#[tokio::test]
async fn test_callback_bounty_done_near() -> testresult::TestResult {
    let (ctx, dao) = setup_dao().await?;
    let alice = create_named_account(&ctx, "alice", 100).await?;

    let proposal_id = propose_bounty_near(&ctx, &dao, &alice).await?;
    vote_approve(&ctx, &ctx.root, &dao, proposal_id).await?;
    claim_bounty_as(&ctx, &dao, &alice, proposal_id).await?;
    done_bounty_as(&ctx, &dao, &alice, proposal_id, &alice).await?;

    // Before the vote there is 1 claim
    let claims_before: u32 = dao
        .call_function("get_bounty_number_of_claims", json!({"id": 0}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(claims_before, 1);

    let alice_balance_before = near_api::Tokens::account(alice.clone())
        .near_balance()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .total;

    // Vote to approve the bounty_done proposal — this triggers the payout
    vote_approve(&ctx, &ctx.root, &dao, proposal_id + 1).await?;

    let alice_balance_after = near_api::Tokens::account(alice.clone())
        .near_balance()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .total;

    let claims_after: u32 = dao
        .call_function("get_bounty_number_of_claims", json!({"id": 0}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(claims_after, 0);
    assert!(
        alice_balance_after > alice_balance_before,
        "alice should have received NEAR payout: before={} after={}",
        alice_balance_before,
        alice_balance_after
    );

    Ok(())
}

#[tokio::test]
async fn test_callback_bounty_done_ft_fail() -> testresult::TestResult {
    let (ctx, dao) = setup_dao().await?;
    let alice = create_named_account(&ctx, "alice", 100).await?;

    // Token without minting to DAO — FT transfer will fail
    let test_token = setup_test_token(&ctx).await?;

    let proposal_id = propose_bounty_ft(&ctx, &dao, &alice, &test_token.0).await?;

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

    // NOTE: alice is NOT registered on the token, so FT transfer will fail
    vote_approve(&ctx, &ctx.root, &dao, proposal_id).await?;
    claim_bounty_as(&ctx, &dao, &alice, proposal_id).await?;
    done_bounty_as(&ctx, &dao, &alice, proposal_id, &alice).await?;
    vote_approve(&ctx, &ctx.root, &dao, proposal_id + 1).await?;

    // The bounty_done proposal should be Failed (FT transfer failed)
    let proposal: ProposalOutput = dao
        .call_function("get_proposal", json!({"id": proposal_id + 1}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(proposal.proposal.status, ProposalStatus::Failed);

    Ok(())
}

#[tokio::test]
async fn test_callback_bounty_done_ft() -> testresult::TestResult {
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

    // Register alice on the token
    near_api::StorageDeposit::on_contract(test_token.0.clone())
        .deposit(alice.clone(), NearToken::from_near(90))
        .with_signer(alice.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .into_result()?;

    // Propose small FT bounty (amount=10)
    let proposal_id: u64 = dao
        .call_function(
            "add_proposal",
            json!({
                "proposal": {
                    "description": "add_new_bounty",
                    "kind": {
                        "AddBounty": {
                            "bounty": {
                                "description": "test_bounties",
                                "token": test_token.0,
                                "amount": "10",
                                "times": 3,
                                "max_deadline": U64(DEADLINE)
                            }
                        }
                    }
                }
            }),
        )
        .transaction()
        .deposit(NearToken::from_near(1))
        .with_signer(alice.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .json()?;

    vote_approve(&ctx, &ctx.root, &dao, proposal_id).await?;
    claim_bounty_as(&ctx, &dao, &alice, proposal_id).await?;
    done_bounty_as(&ctx, &dao, &alice, proposal_id, &alice).await?;

    let claims_before: u32 = dao
        .call_function("get_bounty_number_of_claims", json!({"id": 0}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(claims_before, 1);

    let alice_balance_before = near_api::Tokens::account(alice.clone())
        .near_balance()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .total;

    vote_approve(&ctx, &ctx.root, &dao, proposal_id + 1).await?;

    // Proposal should be approved
    let proposal: ProposalOutput = dao
        .call_function("get_proposal", json!({"id": proposal_id + 1}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(proposal.proposal.status, ProposalStatus::Approved);

    let claims_after: u32 = dao
        .call_function("get_bounty_number_of_claims", json!({"id": 0}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(claims_after, 0);

    // Alice's NEAR balance increased (bond refund)
    let alice_balance_after = near_api::Tokens::account(alice.clone())
        .near_balance()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .total;
    assert!(
        alice_balance_after > alice_balance_before,
        "alice should receive bond refund: before={} after={}",
        alice_balance_before,
        alice_balance_after
    );

    Ok(())
}
