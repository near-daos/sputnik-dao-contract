#![allow(dead_code)]
use near_contract_standards::fungible_token::Balance;
pub use near_sdk::json_types::{Base64VecU8, U64};
use near_sdk::serde_json::{self, json, Value};

use near_workspaces::network::Sandbox;
use near_workspaces::result::ExecutionFinalResult;
use near_workspaces::types::NearToken;
use near_workspaces::{Account, AccountId, Contract, Worker};

use near_sdk::json_types::U128;
use sputnik_staking::Contract as StakingContract;
use sputnikdao2::{
    Action, Bounty, Config, Contract as DAOContract, OldAccountId, ProposalInput, ProposalKind,
    ProposalOutput, ProposalV1, VersionedPolicy, OLD_BASE_TOKEN,
};

pub static FACTORY_WASM_BYTES: &[u8] =
    include_bytes!("../../../sputnikdao-factory2/res/sputnikdao_factory2.wasm");
pub static DAO_WASM_BYTES: &[u8] = include_bytes!("../../res/sputnikdao2.wasm");
pub static TEST_TOKEN_WASM_BYTES: &[u8] = include_bytes!("../../../test-token/res/test_token.wasm");
pub static STAKING_WASM_BYTES: &[u8] =
    include_bytes!("../../../sputnik-staking/res/sputnik_staking.wasm");

pub static SPUTNIKDAO_FACTORY_CONTRACT_ACCOUNT: &str = "sputnik-dao.near";

pub fn base_token() -> Option<near_sdk::AccountId> {
    None
}

pub fn should_fail(r: ExecutionFinalResult) {
    if r.is_success() {
        panic!("Should fail");
    }
}

pub async fn setup_factory() -> Result<(Contract, Worker<Sandbox>), Box<dyn std::error::Error>> {
    let sputnikdao_factory_contract_id: AccountId = SPUTNIKDAO_FACTORY_CONTRACT_ACCOUNT.parse()?;

    let worker = near_workspaces::sandbox().await?;
    let mainnet = near_workspaces::mainnet().await?;

    let _sputnik_dao_factory = worker
        .import_contract(&sputnikdao_factory_contract_id, &mainnet)
        .initial_balance(NearToken::from_near(50))
        .transact()
        .await?;

    let mainnet = near_workspaces::mainnet().await?;
    let sputnikdao_factory_contract_id: AccountId = SPUTNIKDAO_FACTORY_CONTRACT_ACCOUNT.parse()?;

    let worker = near_workspaces::sandbox().await?;

    let sputnik_dao_factory = worker
        .import_contract(&sputnikdao_factory_contract_id, &mainnet)
        .initial_balance(NearToken::from_near(50))
        .transact()
        .await?;

    let deploy_result = sputnik_dao_factory
        .as_account()
        .deploy(FACTORY_WASM_BYTES)
        .await?;
    assert!(deploy_result.is_success());

    let init_sputnik_dao_factory_result =
        sputnik_dao_factory.call("new").max_gas().transact().await?;
    if init_sputnik_dao_factory_result.is_failure() {
        panic!(
            "Error initializing sputnik-dao contract: {:?}",
            String::from_utf8(init_sputnik_dao_factory_result.raw_bytes().unwrap())
        );
    }
    assert!(init_sputnik_dao_factory_result.is_success());
    Ok((sputnik_dao_factory, worker))
}

pub async fn setup_dao() -> Result<(Contract, Worker<Sandbox>, Account), Box<dyn std::error::Error>>
{
    let worker = near_workspaces::sandbox().await?;
    let root = worker.root_account().unwrap();
    setup_dao_with_policy(VersionedPolicy::Default(vec![root.id().clone()])).await
}

pub async fn setup_dao_with_policy(
    policy: VersionedPolicy,
) -> Result<(Contract, Worker<Sandbox>, Account), Box<dyn std::error::Error>> {
    let worker = near_workspaces::sandbox().await?;
    let root = worker.root_account().unwrap();
    let dao_account = root
        .create_subaccount("dao")
        .initial_balance(NearToken::from_near(200))
        .transact()
        .await?
        .result;

    let config = Config {
        name: "test".to_string(),
        purpose: "to test".to_string(),
        metadata: Base64VecU8(vec![]),
    };

    let dao = dao_account.deploy(DAO_WASM_BYTES).await?.result;
    let dao_new_result = dao
        .call("new")
        .args_json(json!({
            "config": config,
            "policy": policy,
        }))
        .max_gas()
        .transact()
        .await?;

    assert!(dao_new_result.is_success());
    Ok((dao, worker, root))
}

pub async fn setup_test_token(root: &Account) -> Result<Contract, Box<dyn std::error::Error>> {
    let test_token_account = root
        .create_subaccount("test_token")
        .initial_balance(NearToken::from_near(200))
        .transact()
        .await?
        .result;

    let test_token_contract = test_token_account
        .deploy(TEST_TOKEN_WASM_BYTES)
        .await?
        .result;

    assert!(test_token_contract
        .call("new")
        .transact()
        .await?
        .is_success());
    Ok(test_token_contract)
}

pub async fn setup_staking(
    root: &Account,
    test_token: &Account,
    dao: &Account,
) -> Result<Contract, Box<dyn std::error::Error>> {
    let staking_account = root
        .create_subaccount("staking")
        .initial_balance(NearToken::from_near(100))
        .transact()
        .await?
        .result;

    let staking_contract = staking_account.deploy(STAKING_WASM_BYTES).await?.result;

    assert!(staking_contract
        .call("new")
        .args_json(json!({
            "owner_id": dao.id(),
            "token_id": test_token.id(),
            "unstake_period": U64(100_000_000_000)
        }))
        .transact()
        .await?
        .is_success());
    Ok(staking_contract)
}

pub async fn add_proposal(dao: &Contract, proposal: ProposalInput) -> ExecutionFinalResult {
    dao.call("add_proposal")
        .args_json(json!({"proposal": proposal}))
        .deposit(NearToken::from_near(1))
        .transact()
        .await
        .unwrap()
}

pub async fn add_member_proposal(
    dao: &Contract,
    member_id: near_sdk::AccountId,
) -> ExecutionFinalResult {
    add_proposal(
        dao,
        ProposalInput {
            description: "test".to_string(),
            kind: ProposalKind::AddMemberToRole {
                member_id: member_id,
                role: "council".to_string(),
            },
        },
    )
    .await
}

pub async fn add_transfer_proposal(
    dao: &Contract,
    token_id: Option<near_sdk::AccountId>,
    receiver_id: near_sdk::AccountId,
    amount: Balance,
    msg: Option<String>,
) -> ExecutionFinalResult {
    add_proposal(
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

pub async fn add_bounty_proposal(worker: &Worker<Sandbox>, dao: &Contract) -> ExecutionFinalResult {
    add_proposal(
        dao,
        ProposalInput {
            description: "test".to_string(),
            kind: ProposalKind::AddBounty {
                bounty: Bounty {
                    description: "test bounty".to_string(),
                    token: String::from(OLD_BASE_TOKEN),
                    amount: U128(NearToken::from_near(10).as_yoctonear()),
                    times: 3,
                    max_deadline: U64(
                        worker.view_block().await.unwrap().timestamp() + 10_000_000_000
                    ),
                },
            },
        },
    )
    .await
}

pub async fn vote(
    users: Vec<&Account>,
    dao: &Contract,
    proposal_id: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    for user in users.into_iter() {
        let act_proposal_result = user
            .call(dao.id(), "act_proposal")
            .args_json(json!({
                "id": proposal_id,
                "action": Action::VoteApprove,
                "proposal": get_proposal_kind(&dao, proposal_id).await}))
            .max_gas()
            .transact()
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

pub async fn get_proposal_kind(dao: &Contract, proposal_id: u64) -> ProposalKind {
    dao.view("get_proposal")
        .args_json(json!({"id": proposal_id}))
        .await
        .unwrap()
        .json::<ProposalOutput>()
        .unwrap()
        .proposal
        .latest_version()
        .kind
}
