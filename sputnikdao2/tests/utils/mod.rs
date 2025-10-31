#![allow(dead_code)]
use std::sync::Arc;

use near_contract_standards::fungible_token::Balance;
use near_sandbox::{
    config::{DEFAULT_GENESIS_ACCOUNT, DEFAULT_GENESIS_ACCOUNT_PRIVATE_KEY},
    Sandbox,
};
pub use near_sdk::json_types::{Base64VecU8, U64};
use near_sdk::{serde_json::json, AccountIdRef};

use near_api::{
    types::{
        transaction::result::ExecutionFinalResult, AccessKey, AccessKeyPermission, AccountId,
        NearToken, TxExecutionStatus,
    },
    Contract, Signer,
};

use near_sdk::json_types::U128;
use sputnikdao2::{
    Action, Bounty, Config, OldAccountId, ProposalInput, ProposalKind, ProposalOutput,
    VersionedPolicy, OLD_BASE_TOKEN,
};

pub static FACTORY_WASM_BYTES: &[u8] =
    include_bytes!("../../../sputnikdao-factory2/res/sputnikdao_factory2.wasm");
pub static DAO_WASM_BYTES: &[u8] = include_bytes!("../../res/sputnikdao2.wasm");
pub static TEST_TOKEN_WASM_BYTES: &[u8] = include_bytes!("../../../test-token/res/test_token.wasm");
pub static STAKING_WASM_BYTES: &[u8] =
    include_bytes!("../../../sputnik-staking/res/sputnik_staking.wasm");

pub static SPUTNIKDAO_FACTORY_CONTRACT_ACCOUNT: &AccountIdRef =
    AccountIdRef::new_or_panic("sputnik-dao.near");

pub fn base_token() -> Option<near_sdk::AccountId> {
    None
}

pub fn should_fail(r: ExecutionFinalResult) {
    if r.is_success() {
        panic!("Should fail");
    }
}

pub struct TestContext {
    pub sandbox: Sandbox,
    pub sandbox_network: near_api::NetworkConfig,
    pub signer: Arc<Signer>,
    pub root: AccountId,
}

pub async fn setup_factory() -> Result<(TestContext, Contract), Box<dyn std::error::Error>> {
    let sputnikdao_factory_contract_id: AccountId = SPUTNIKDAO_FACTORY_CONTRACT_ACCOUNT.into();

    let sandbox = near_sandbox::Sandbox::start_sandbox().await?;
    let sandbox_network =
        near_api::NetworkConfig::from_rpc_url("sandbox", sandbox.rpc_addr.parse()?);

    let private_key = near_api::signer::generate_secret_key()?;

    let mut sputnik_dao_factory_account = near_api::Account(sputnikdao_factory_contract_id.clone())
        .view()
        .fetch_from_mainnet()
        .await?
        .data;
    sputnik_dao_factory_account.amount = NearToken::from_near(50);
    let sputnik_dao_code = near_api::Contract(sputnikdao_factory_contract_id.clone())
        .wasm()
        .fetch_from_mainnet()
        .await?
        .data;

    sandbox
        .patch_state(sputnikdao_factory_contract_id.clone())
        .account(sputnik_dao_factory_account)
        .code(sputnik_dao_code.code_base64)
        .access_key(
            private_key.public_key().to_string(),
            AccessKey {
                nonce: 0.into(),
                permission: AccessKeyPermission::FullAccess,
            },
        )
        .send()
        .await?;

    let signer = Signer::new(Signer::from_secret_key(private_key))?;

    let deploy_result = Contract::deploy(sputnikdao_factory_contract_id.clone())
        .use_code(FACTORY_WASM_BYTES.to_vec())
        .with_init_call("new", ())?
        .max_gas()
        .with_signer(signer.clone())
        .send_to(&sandbox_network)
        .await?;

    assert!(deploy_result.is_success());

    Ok((
        TestContext {
            sandbox,
            sandbox_network,
            signer,
            root: sputnikdao_factory_contract_id.clone(),
        },
        Contract(sputnikdao_factory_contract_id),
    ))
}

pub async fn setup_dao() -> Result<(TestContext, Contract), Box<dyn std::error::Error>> {
    let sandbox = near_sandbox::Sandbox::start_sandbox().await?;
    let root_account = DEFAULT_GENESIS_ACCOUNT.to_owned();
    let signer = Signer::new(Signer::from_secret_key(
        DEFAULT_GENESIS_ACCOUNT_PRIVATE_KEY.parse()?,
    ))?;

    setup_dao_with_params(
        root_account.clone(),
        signer,
        sandbox,
        VersionedPolicy::Default(vec![root_account.clone()]),
    )
    .await
}

pub async fn setup_dao_with_params(
    root: AccountId,
    signer: Arc<Signer>,
    sandbox: Sandbox,
    policy: VersionedPolicy,
) -> Result<(TestContext, Contract), Box<dyn std::error::Error>> {
    let dao_account_id: AccountId = format!("dao.{root}").parse().unwrap();
    let sandbox_network =
        near_api::NetworkConfig::from_rpc_url("sandbox", sandbox.rpc_addr.parse()?);

    near_api::Account::create_account(dao_account_id.clone())
        .fund_myself(root.clone(), NearToken::from_near(200))
        .public_key(signer.get_public_key().await?)
        .unwrap()
        .with_signer(signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    let config = Config {
        name: "test".to_string(),
        purpose: "to test".to_string(),
        metadata: Base64VecU8(vec![]),
    };

    near_api::Contract::deploy(dao_account_id.clone())
        .use_code(DAO_WASM_BYTES.to_vec())
        .with_init_call(
            "new",
            json!({
                "config": config,
                "policy": policy,
            }),
        )?
        .max_gas()
        .with_signer(signer.clone())
        .send_to(&sandbox_network)
        .await?
        .assert_success();

    Ok((
        TestContext {
            sandbox,
            sandbox_network,
            signer,
            root,
        },
        Contract(dao_account_id),
    ))
}

pub async fn setup_test_token(ctx: &TestContext) -> Result<Contract, Box<dyn std::error::Error>> {
    let test_token_account_id: AccountId = format!("test_token.{}", ctx.root).parse().unwrap();
    near_api::Account::create_account(test_token_account_id.clone())
        .fund_myself(ctx.root.clone(), NearToken::from_near(200))
        .public_key(ctx.signer.get_public_key().await?)
        .unwrap()
        .with_signer(ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .assert_success();

    near_api::Contract::deploy(test_token_account_id.clone())
        .use_code(TEST_TOKEN_WASM_BYTES.to_vec())
        .with_init_call("new", ())?
        .max_gas()
        .with_signer(ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .assert_success();

    Ok(Contract(test_token_account_id))
}

pub async fn setup_staking(
    ctx: &TestContext,
    test_token: &AccountId,
    dao: &AccountId,
) -> Result<Contract, Box<dyn std::error::Error>> {
    let staking_account_id: AccountId = format!("staking.{}", ctx.root).parse().unwrap();
    near_api::Account::create_account(staking_account_id.clone())
        .fund_myself(ctx.root.clone(), NearToken::from_near(100))
        .public_key(ctx.signer.get_public_key().await?)
        .unwrap()
        .with_signer(ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .assert_success();

    near_api::Contract::deploy(staking_account_id.clone())
        .use_code(STAKING_WASM_BYTES.to_vec())
        .with_init_call(
            "new",
            json!({
                "owner_id": dao,
                "token_id": test_token,
                "unstake_period": U64(100_000_000_000)
            }),
        )?
        .max_gas()
        .with_signer(ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .assert_success();

    Ok(Contract(staking_account_id))
}

pub async fn add_proposal(
    ctx: &TestContext,
    dao: &Contract,
    proposal: ProposalInput,
) -> ExecutionFinalResult {
    dao.call_function("add_proposal", json!({"proposal": proposal}))
        .unwrap()
        .transaction()
        .deposit(NearToken::from_near(1))
        .with_signer(dao.0.clone(), ctx.signer.clone())
        .wait_until(TxExecutionStatus::ExecutedOptimistic)
        .send_to(&ctx.sandbox_network)
        .await
        .unwrap()
}

pub async fn add_member_proposal(
    ctx: &TestContext,
    dao: &Contract,
    member_id: near_sdk::AccountId,
) -> ExecutionFinalResult {
    add_proposal(
        ctx,
        dao,
        ProposalInput {
            description: "test".to_string(),
            kind: ProposalKind::AddMemberToRole {
                member_id,
                role: "council".to_string(),
            },
        },
    )
    .await
}

pub async fn add_transfer_proposal(
    ctx: &TestContext,
    dao: &Contract,
    token_id: Option<near_sdk::AccountId>,
    receiver_id: near_sdk::AccountId,
    amount: Balance,
    msg: Option<String>,
) -> ExecutionFinalResult {
    add_proposal(
        ctx,
        dao,
        ProposalInput {
            description: "test".to_string(),
            kind: ProposalKind::Transfer {
                token_id: convert_new_to_old_token(token_id),
                receiver_id,
                amount: U128(amount),
                msg,
            },
        },
    )
    .await
}

pub async fn add_bounty_proposal(ctx: &TestContext, dao: &Contract) -> ExecutionFinalResult {
    let block_timestamp = near_api::Chain::block()
        .fetch_from(&ctx.sandbox_network)
        .await
        .unwrap()
        .header
        .timestamp;
    add_proposal(
        ctx,
        dao,
        ProposalInput {
            description: "test".to_string(),
            kind: ProposalKind::AddBounty {
                bounty: Bounty {
                    description: "test bounty".to_string(),
                    token: String::from(OLD_BASE_TOKEN),
                    amount: U128(NearToken::from_near(10).as_yoctonear()),
                    times: 3,
                    max_deadline: U64(block_timestamp + 10_000_000_000),
                },
            },
        },
    )
    .await
}

pub async fn vote(
    ctx: &TestContext,
    users: Vec<&AccountId>,
    dao: &Contract,
    proposal_id: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    for user in users.into_iter() {
        let act_proposal_result = dao
            .call_function(
                "act_proposal",
                json!({
                    "id": proposal_id,
                    "action": Action::VoteApprove,
                    "proposal": get_proposal_kind(ctx, dao, proposal_id).await
                }),
            )
            .unwrap()
            .transaction()
            .max_gas()
            .with_signer(user.clone(), ctx.signer.clone())
            .send_to(&ctx.sandbox_network)
            .await?;
        assert!(
            act_proposal_result.is_success(),
            "{:?}",
            act_proposal_result.failures()
        );
    }
    Ok(())
}

pub fn convert_new_to_old_token(new_account_id: Option<near_sdk::AccountId>) -> OldAccountId {
    if new_account_id.is_none() {
        return String::from(OLD_BASE_TOKEN);
    }
    new_account_id.unwrap().to_string()
}

pub async fn get_proposal_kind(
    ctx: &TestContext,
    dao: &Contract,
    proposal_id: u64,
) -> ProposalKind {
    dao.call_function("get_proposal", json!({"id": proposal_id}))
        .unwrap()
        .read_only::<ProposalOutput>()
        .fetch_from(&ctx.sandbox_network)
        .await
        .unwrap()
        .data
        .proposal
        .kind
}
