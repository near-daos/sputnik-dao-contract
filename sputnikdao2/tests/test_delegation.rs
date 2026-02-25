use near_sdk::json_types::U128;
use near_sdk::serde_json::json;

use near_api::NearToken;

mod utils;
use crate::utils::*;

#[tokio::test]
async fn test_register_delegation() -> testresult::TestResult {
    let (ctx, dao) = setup_dao().await?;
    let alice = create_named_account(&ctx, "alice", 100).await?;

    let test_token = setup_test_token(&ctx).await?;
    let staking = setup_staking(&ctx, &test_token.0, &dao.0).await?;

    set_staking_contract(&ctx, &dao, &staking.0).await?;

    register_and_delegate(&ctx, &dao, &staking, &alice, 1).await?;

    let balance: U128 = dao
        .call_function("delegation_balance_of", json!({"account_id": alice}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(balance, U128(1));

    let total: U128 = dao
        .call_function("delegation_total_supply", json!({}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(total, U128(1));

    Ok(())
}

#[tokio::test]
async fn test_register_delegation_fail() -> testresult::TestResult {
    let (ctx, dao) = setup_dao().await?;
    let alice = create_named_account(&ctx, "alice", 100).await?;

    let test_token = setup_test_token(&ctx).await?;
    let staking = setup_staking(&ctx, &test_token.0, &dao.0).await?;

    // Staking id not set yet → ERR_NO_STAKING
    let result = dao
        .call_function("register_delegation", json!({"account_id": alice}))
        .transaction()
        .deposit(NearToken::from_yoctonear(REG_COST))
        .with_signer(staking.0.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        format!("{:?}", result.failures()).contains("ERR_NO_STAKING"),
        "{:?}",
        result.failures()
    );

    set_staking_contract(&ctx, &dao, &staking.0).await?;

    // Can only be called by the staking_id — root calling → ERR_INVALID_CALLER
    let result = dao
        .call_function("register_delegation", json!({"account_id": alice}))
        .transaction()
        .deposit(NearToken::from_yoctonear(REG_COST))
        .with_signer(ctx.root.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        format!("{:?}", result.failures()).contains("ERR_INVALID_CALLER"),
        "{:?}",
        result.failures()
    );

    Ok(())
}

#[tokio::test]
async fn test_delegation() -> testresult::TestResult {
    let (ctx, dao) = setup_dao().await?;
    let alice = create_named_account(&ctx, "alice", 100).await?;
    let bob = create_named_account(&ctx, "bob", 100).await?;

    let test_token = setup_test_token(&ctx).await?;
    let staking = setup_staking(&ctx, &test_token.0, &dao.0).await?;

    set_staking_contract(&ctx, &dao, &staking.0).await?;

    let random_amount: u128 = 10_087_687_667_869;

    let (old, new_bal, total) =
        register_and_delegate(&ctx, &dao, &staking, &alice, random_amount).await?;
    assert_eq!(old, U128(0));
    assert_eq!(new_bal, U128(random_amount));
    assert_eq!(total, U128(random_amount));

    let (old, new_bal, total) =
        register_and_delegate(&ctx, &dao, &staking, &bob, random_amount * 2).await?;
    assert_eq!(old, U128(0));
    assert_eq!(new_bal, U128(random_amount * 2));
    assert_eq!(total, U128(random_amount * 3));

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

    let total_supply: U128 = dao
        .call_function("delegation_total_supply", json!({}))
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(total_supply, U128(random_amount * 3));

    Ok(())
}

#[tokio::test]
async fn test_delegation_fail() -> testresult::TestResult {
    let (ctx, dao) = setup_dao().await?;
    let alice = create_named_account(&ctx, "alice", 100).await?;

    let test_token = setup_test_token(&ctx).await?;
    let staking = setup_staking(&ctx, &test_token.0, &dao.0).await?;

    let random_amount: u128 = 10_087_687_667_869;

    // staking_id is None → ERR_NO_STAKING
    let result = dao
        .call_function(
            "delegate",
            json!({"account_id": alice, "amount": U128(random_amount)}),
        )
        .transaction()
        .with_signer(staking.0.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        format!("{:?}", result.failures()).contains("ERR_NO_STAKING"),
        "{:?}",
        result.failures()
    );

    set_staking_contract(&ctx, &dao, &staking.0).await?;

    // Non-staking caller → ERR_INVALID_CALLER
    let result = dao
        .call_function(
            "delegate",
            json!({"account_id": alice, "amount": U128(random_amount)}),
        )
        .transaction()
        .with_signer(ctx.root.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        format!("{:?}", result.failures()).contains("ERR_INVALID_CALLER"),
        "{:?}",
        result.failures()
    );

    // Account not registered → ERR_NOT_REGISTERED
    let not_registered: near_api::AccountId = "not-registered.sandbox".parse()?;
    let result = dao
        .call_function(
            "delegate",
            json!({"account_id": not_registered, "amount": U128(random_amount)}),
        )
        .transaction()
        .with_signer(staking.0.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        format!("{:?}", result.failures()).contains("ERR_NOT_REGISTERED"),
        "{:?}",
        result.failures()
    );

    Ok(())
}

#[tokio::test]
async fn test_undelegate() -> testresult::TestResult {
    let (ctx, dao) = setup_dao().await?;
    let alice = create_named_account(&ctx, "alice", 100).await?;

    let test_token = setup_test_token(&ctx).await?;
    let staking = setup_staking(&ctx, &test_token.0, &dao.0).await?;

    set_staking_contract(&ctx, &dao, &staking.0).await?;

    let random_amount: u128 = 44_887_687_667_868;

    register_and_delegate(&ctx, &dao, &staking, &alice, random_amount).await?;

    let result: (U128, U128, U128) = dao
        .call_function(
            "undelegate",
            json!({
                "account_id": alice,
                "amount": U128(random_amount / 2)
            }),
        )
        .transaction()
        .with_signer(staking.0.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .json()?;

    assert_eq!(result.0, U128(random_amount));
    assert_eq!(result.1, U128(random_amount - random_amount / 2));
    assert_eq!(result.2, U128(random_amount - random_amount / 2));

    Ok(())
}

#[tokio::test]
async fn test_undelegate_fail() -> testresult::TestResult {
    let (ctx, dao) = setup_dao().await?;
    let alice = create_named_account(&ctx, "alice", 100).await?;

    let test_token = setup_test_token(&ctx).await?;
    let staking = setup_staking(&ctx, &test_token.0, &dao.0).await?;

    let random_amount: u128 = 44_887_687_667_868;

    // staking_id is None → ERR_NO_STAKING
    let result = dao
        .call_function(
            "undelegate",
            json!({"account_id": alice, "amount": U128(random_amount)}),
        )
        .transaction()
        .with_signer(staking.0.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        format!("{:?}", result.failures()).contains("ERR_NO_STAKING"),
        "{:?}",
        result.failures()
    );

    set_staking_contract(&ctx, &dao, &staking.0).await?;

    // Non-staking caller → ERR_INVALID_CALLER
    let result = dao
        .call_function(
            "undelegate",
            json!({"account_id": alice, "amount": U128(random_amount)}),
        )
        .transaction()
        .with_signer(ctx.root.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        format!("{:?}", result.failures()).contains("ERR_INVALID_CALLER"),
        "{:?}",
        result.failures()
    );

    // Register alice and delegate
    register_and_delegate(&ctx, &dao, &staking, &alice, random_amount).await?;

    // Trying to undelegate more than delegated → ERR_INVALID_STAKING_CONTRACT
    let result = dao
        .call_function(
            "undelegate",
            json!({
                "account_id": alice,
                "amount": U128(random_amount + 1)
            }),
        )
        .transaction()
        .with_signer(staking.0.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        format!("{:?}", result.failures()).contains("ERR_INVALID_STAKING_CONTRACT"),
        "{:?}",
        result.failures()
    );

    Ok(())
}
