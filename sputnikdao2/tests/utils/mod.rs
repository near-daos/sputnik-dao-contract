#![allow(dead_code)]
// std::cell::OnceCell is not Sync and cannot be placed in a static; std::sync::OnceLock is its
// thread-safe counterpart and is the correct choice for lazy-initialised statics in async tests.
use std::sync::OnceLock;

use near_contract_standards::fungible_token::Balance;
use near_sandbox::{
    Sandbox,
    config::{DEFAULT_GENESIS_ACCOUNT, DEFAULT_GENESIS_ACCOUNT_PRIVATE_KEY},
};
pub use near_sdk::json_types::{Base64VecU8, U64};
use near_sdk::{AccountIdRef, serde_json::json};

use near_api::{
    Contract, RPCEndpoint, Signer,
    types::{AccountId, NearToken, TxExecutionStatus, transaction::result::ExecutionFinalResult},
};

use near_sdk::json_types::U128;
use sputnikdao2::{
    Action, Bounty, Config, OLD_BASE_TOKEN, OldAccountId, ProposalInput, ProposalKind,
    ProposalOutput, VersionedPolicy,
};

// ---------------------------------------------------------------------------
// On-demand WASM builds, cached per process via OnceLock.
// Each contract is compiled exactly once regardless of how many tests run.
// ---------------------------------------------------------------------------

static DAO_WASM: OnceLock<Vec<u8>> = OnceLock::new();
static FACTORY_WASM: OnceLock<Vec<u8>> = OnceLock::new();
static TEST_TOKEN_WASM: OnceLock<Vec<u8>> = OnceLock::new();
static STAKING_WASM: OnceLock<Vec<u8>> = OnceLock::new();

/// Build sputnikdao2 (the current package) and return its WASM bytes.
pub fn dao_wasm_bytes() -> &'static [u8] {
    DAO_WASM.get_or_init(|| {
        let wasm_path = cargo_near_build::build_with_cli(Default::default())
            .expect("Failed to build sputnikdao2");
        std::fs::read(&wasm_path).expect("Failed to read sputnikdao2.wasm")
    })
}

/// Build sputnikdao-factory2 and return its WASM bytes.
pub fn factory_wasm_bytes() -> &'static [u8] {
    FACTORY_WASM.get_or_init(|| {
        let wasm_path = cargo_near_build::build_with_cli(cargo_near_build::BuildOpts {
            manifest_path: Some(
                concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../sputnikdao-factory2/Cargo.toml"
                )
                .into(),
            ),
            ..Default::default()
        })
        .expect("Failed to build sputnikdao-factory2");
        std::fs::read(&wasm_path).expect("Failed to read sputnikdao_factory2.wasm")
    })
}

/// Build test-token and return its WASM bytes.
pub fn test_token_wasm_bytes() -> &'static [u8] {
    TEST_TOKEN_WASM.get_or_init(|| {
        let wasm_path = cargo_near_build::build_with_cli(cargo_near_build::BuildOpts {
            manifest_path: Some(
                concat!(env!("CARGO_MANIFEST_DIR"), "/../test-token/Cargo.toml").into(),
            ),
            ..Default::default()
        })
        .expect("Failed to build test-token");
        std::fs::read(&wasm_path).expect("Failed to read test_token.wasm")
    })
}

/// Build sputnik-staking and return its WASM bytes.
pub fn staking_wasm_bytes() -> &'static [u8] {
    STAKING_WASM.get_or_init(|| {
        let wasm_path = cargo_near_build::build_with_cli(cargo_near_build::BuildOpts {
            manifest_path: Some(
                concat!(env!("CARGO_MANIFEST_DIR"), "/../sputnik-staking/Cargo.toml").into(),
            ),
            ..Default::default()
        })
        .expect("Failed to build sputnik-staking");
        std::fs::read(&wasm_path).expect("Failed to read sputnik_staking.wasm")
    })
}

// ---------------------------------------------------------------------------
// Test infrastructure
// ---------------------------------------------------------------------------

pub static SPUTNIKDAO_FACTORY_CONTRACT_ACCOUNT: &AccountIdRef =
    AccountIdRef::new_or_panic("sputnik-dao.near");

pub fn base_token() -> Option<near_sdk::AccountId> {
    None
}

pub struct TestContext {
    pub sandbox: Sandbox,
    pub sandbox_network: near_api::NetworkConfig,
    pub signer: std::sync::Arc<Signer>,
    pub root: AccountId,
}

pub async fn setup_factory() -> Result<(TestContext, Contract), Box<dyn std::error::Error>> {
    let sandbox = near_sandbox::Sandbox::start_sandbox().await?;
    let sandbox_network =
        near_api::NetworkConfig::from_rpc_url("sandbox", sandbox.rpc_addr.parse()?);

    sandbox
        .import_account(
            RPCEndpoint::mainnet().url,
            SPUTNIKDAO_FACTORY_CONTRACT_ACCOUNT.to_owned(),
        )
        .initial_balance(NearToken::from_near(100))
        .send()
        .await?;

    let signer = Signer::from_secret_key(DEFAULT_GENESIS_ACCOUNT_PRIVATE_KEY.parse()?)?;

    let deploy_result = Contract::deploy(SPUTNIKDAO_FACTORY_CONTRACT_ACCOUNT.to_owned())
        .use_code(factory_wasm_bytes().to_vec())
        .with_init_call("new", ())?
        .max_gas()
        .with_signer(signer.clone())
        .send_to(&sandbox_network)
        .await?;

    assert!(deploy_result.is_success());

    let sputnikdao_factory_contract = Contract(SPUTNIKDAO_FACTORY_CONTRACT_ACCOUNT.to_owned());

    // The factory starts with no default code hash, so we store the DAO wasm and register it.
    let dao_hash: near_sdk::json_types::Base58CryptoHash = sputnikdao_factory_contract
        .call_function_raw("store", dao_wasm_bytes().to_vec())
        .transaction()
        .max_gas()
        .deposit(NearToken::from_near(50))
        .with_signer(
            SPUTNIKDAO_FACTORY_CONTRACT_ACCOUNT.to_owned(),
            signer.clone(),
        )
        .send_to(&sandbox_network)
        .await?
        .json()?;
    sputnikdao_factory_contract
        .call_function("set_default_code_hash", json!({ "code_hash": dao_hash }))
        .transaction()
        .with_signer(
            SPUTNIKDAO_FACTORY_CONTRACT_ACCOUNT.to_owned(),
            signer.clone(),
        )
        .send_to(&sandbox_network)
        .await?
        .into_result()?;

    Ok((
        TestContext {
            sandbox,
            sandbox_network,
            signer,
            root: SPUTNIKDAO_FACTORY_CONTRACT_ACCOUNT.to_owned(),
        },
        sputnikdao_factory_contract,
    ))
}

pub async fn setup_dao() -> testresult::TestResult<(TestContext, Contract)> {
    let sandbox = near_sandbox::Sandbox::start_sandbox().await?;
    let root_account = DEFAULT_GENESIS_ACCOUNT.to_owned();
    let signer = Signer::from_secret_key(DEFAULT_GENESIS_ACCOUNT_PRIVATE_KEY.parse()?)?;

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
    signer: std::sync::Arc<Signer>,
    sandbox: Sandbox,
    policy: VersionedPolicy,
) -> testresult::TestResult<(TestContext, Contract)> {
    let dao_account_id: AccountId = format!("dao.{root}").parse()?;
    let sandbox_network =
        near_api::NetworkConfig::from_rpc_url("sandbox", sandbox.rpc_addr.parse()?);

    sandbox
        .create_account(dao_account_id.clone())
        .initial_balance(NearToken::from_near(200))
        .send()
        .await?;

    let config = Config {
        name: "test".to_string(),
        purpose: "to test".to_string(),
        metadata: Base64VecU8(vec![]),
    };

    near_api::Contract::deploy(dao_account_id.clone())
        .use_code(dao_wasm_bytes().to_vec())
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
        .into_result()?;

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

pub async fn setup_test_token(ctx: &TestContext) -> testresult::TestResult<Contract> {
    let test_token_account_id: AccountId = format!("test_token.{}", ctx.root).parse().unwrap();
    ctx.sandbox
        .create_account(test_token_account_id.clone())
        .initial_balance(NearToken::from_near(200))
        .send()
        .await?;

    near_api::Contract::deploy(test_token_account_id.clone())
        .use_code(test_token_wasm_bytes().to_vec())
        .with_init_call("new", ())?
        .max_gas()
        .with_signer(ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .into_result()?;

    Ok(Contract(test_token_account_id))
}

pub async fn setup_staking(
    ctx: &TestContext,
    test_token: &AccountId,
    dao: &AccountId,
) -> testresult::TestResult<Contract> {
    let staking_account_id: AccountId = format!("staking.{}", ctx.root).parse().unwrap();
    ctx.sandbox
        .create_account(staking_account_id.clone())
        .initial_balance(NearToken::from_near(100))
        .send()
        .await?;

    near_api::Contract::deploy(staking_account_id.clone())
        .use_code(staking_wasm_bytes().to_vec())
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
        .into_result()?;

    Ok(Contract(staking_account_id))
}

pub async fn add_proposal(
    ctx: &TestContext,
    dao: &Contract,
    proposal: ProposalInput,
) -> ExecutionFinalResult {
    dao.call_function("add_proposal", json!({"proposal": proposal}))
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
) -> testresult::TestResult {
    for user in users.into_iter() {
        dao.call_function(
            "act_proposal",
            json!({
                "id": proposal_id,
                "action": Action::VoteApprove,
                "proposal": get_proposal_kind(ctx, dao, proposal_id).await?
            }),
        )
        .transaction()
        .max_gas()
        .with_signer(user.clone(), ctx.signer.clone())
        .send_to(&ctx.sandbox_network)
        .await?
        .into_result()?;
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
) -> testresult::TestResult<ProposalKind> {
    Ok(dao
        .call_function("get_proposal", json!({"id": proposal_id}))
        .read_only::<ProposalOutput>()
        .fetch_from(&ctx.sandbox_network)
        .await?
        .data
        .proposal
        .kind)
}
