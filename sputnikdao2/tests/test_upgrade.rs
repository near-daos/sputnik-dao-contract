use near_sdk::json_types::Base58CryptoHash;
use near_sdk::serde_json::json;

use near_api::AccountId;
use near_api::NearToken;
use sputnikdao2::{Action, Config, ProposalInput, ProposalKind, VersionedPolicy};

use rand::Rng;
use walrus::ModuleConfig;

mod utils;
use crate::utils::*;

pub static DAO_WASM_BYTES: &[u8] = include_bytes!("../res/sputnikdao2.wasm");
pub static OTHER_WASM_BYTES: &[u8] = include_bytes!("../res/ref_exchange_release.wasm");

pub fn add_data_segment(wasm: &[u8], size: usize) -> anyhow::Result<Vec<u8>> {
    let mut module = ModuleConfig::new().parse(wasm)?;

    let random_data: Vec<u8> = (0..size).map(|_| rand::thread_rng().gen()).collect();
    let data_id = module.data.add(walrus::DataKind::Passive, random_data);

    module.data.get_mut(data_id).name = Some("zero-padding".to_string());

    let new_bytes = module.emit_wasm();
    Ok(new_bytes)
}

#[tokio::test]
async fn test_upgrade_using_factory() -> Result<(), Box<dyn std::error::Error>> {
    let (ctx, factory) = setup_factory().await?;
    let root = ctx.root;

    let config = Config {
        name: "testdao".to_string(),
        purpose: "to test".to_string(),
        metadata: Base64VecU8(vec![]),
    };
    let policy = VersionedPolicy::Default(vec![root.clone()]);
    let params = json!({ "config": config, "policy": policy })
        .to_string()
        .into_bytes();

    let create_result = factory
        .call_function(
            "create",
            json!({
                "name": "testdao",
                "args": Base64VecU8(params)
            }),
        )?
        .transaction()
        .deposit(NearToken::from_near(10))
        .max_gas()
        .with_signer(root.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;

    assert!(create_result.is_success(), "{:?}", create_result.failures());

    let dao_account_id: AccountId = format!("testdao.{}", factory.0).parse()?;
    let dao_list: Vec<near_api::AccountId> = factory
        .call_function("get_dao_list", ())?
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;
    assert_eq!(dao_list, vec![dao_account_id.clone()]);

    let hash: Base58CryptoHash = factory
        .call_function("get_default_code_hash", ())?
        .read_only()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data;

    let proposal_kind = ProposalKind::UpgradeSelf { hash };
    let proposal_id: u64 = near_api::Contract(dao_account_id.clone())
        .call_function(
            "add_proposal",
            json!({ "proposal": ProposalInput {
                description: "proposal to test".to_string(),
                kind: proposal_kind.clone()
            }}),
        )?
        .transaction()
        .deposit(NearToken::from_near(1))
        .with_signer(root.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .assert_success()
        .json()?;

    assert_eq!(0, proposal_id);

    let act_proposal_result = near_api::Contract(dao_account_id.clone())
        .call_function(
            "act_proposal",
            json!({
                "id": 0,
                "action": Action::VoteApprove,
                "proposal": proposal_kind
            }),
        )?
        .transaction()
        .with_signer(root.clone(), ctx.signer)
        .send_to(&ctx.sandbox_network)
        .await?;
    assert!(
        act_proposal_result.is_success(),
        "{:?}",
        act_proposal_result.failures()
    );
    assert_eq!(
        0,
        act_proposal_result.failures().len(),
        "{:?}",
        act_proposal_result.failures()
    );

    Ok(())
}

/// Test that Sputnik can upgrade another contract.
#[tokio::test]
async fn test_upgrade_other() -> Result<(), Box<dyn std::error::Error>> {
    let (ctx, dao) = setup_dao().await?;

    let ref_account: AccountId = format!("ref-finance.{}", ctx.root).parse().unwrap();
    ctx.sandbox
        .create_account(ref_account.clone())
        .initial_balance(NearToken::from_near(2000))
        .send()
        .await?;

    let ref_contract_new_result = near_api::Contract::deploy(ref_account.clone())
        .use_code(OTHER_WASM_BYTES.to_vec())
        .with_init_call(
            "new",
            json!({
                "owner_id": dao.0,
                "exchange_fee": 1,
                "referral_fee": 1,
            }),
        )?
        .with_signer(ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?;

    assert!(
        ref_contract_new_result.is_success(),
        "{:?}",
        ref_contract_new_result.failures()
    );

    let extended_wasm = add_data_segment(OTHER_WASM_BYTES, 1200 * 1024)?;
    assert_eq!(extended_wasm.len(), 1566669);

    let hash = dao
        .call_function_raw("store_blob", extended_wasm)
        .transaction()
        .deposit(NearToken::from_near(200))
        .max_gas()
        .with_signer(ctx.root.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .assert_success()
        .json()?;

    assert!(add_proposal(
        &ctx,
        &dao,
        ProposalInput {
            description: "test".to_string(),
            kind: ProposalKind::UpgradeRemote {
                receiver_id: ref_account.clone(),
                method_name: "upgrade".to_string(),
                hash,
            },
        },
    )
    .await
    .is_success());

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

    assert_eq!(
        0,
        act_proposal_result.failures().len(),
        "{:?}",
        act_proposal_result.failures()
    );
    assert!(act_proposal_result.is_success());

    Ok(())
}
