#![allow(dead_code)]
pub use near_sdk::json_types::{Base64VecU8, U64};

use near_workspaces::network::Sandbox;
use near_workspaces::result::ExecutionFinalResult;
use near_workspaces::types::NearToken;
use near_workspaces::{AccountId, Contract, Worker};

use near_sdk::json_types::U128;
use sputnik_staking::ContractContract as StakingContract;
use sputnikdao2::{
    Action, Bounty, Config, ContractContract as DAOContract, OldAccountId, ProposalInput,
    ProposalKind, VersionedPolicy, OLD_BASE_TOKEN,
};

pub static FACTORY_WASM_BYTES: &[u8] =
    include_bytes!("../../../sputnikdao-factory2/res/sputnikdao_factory2.wasm");
pub static DAO_WASM_BYTES: &[u8] = include_bytes!("../../res/sputnikdao2.wasm");
pub static TEST_TOKEN_WASM_BYTES: &[u8] = include_bytes!("../../../test-token/res/test_token.wasm");
pub static STAKING_WASM_BYTES: &[u8] =
    include_bytes!("../../../sputnik-staking/res/sputnik_staking.wasm");

pub static SPUTNIKDAO_FACTORY_CONTRACT_ACCOUNT: &str = "sputnik-dao.near";

pub fn root() -> near_sdk::AccountId {
    near_sdk::AccountId::new_unchecked("near".to_string())
}

pub fn base_token() -> Option<AccountId> {
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

/*

pub fn setup_dao() -> (UserAccount, Contract) {
    let root = init_simulator(None);
    let config = Config {
        name: "test".to_string(),
        purpose: "to test".to_string(),
        metadata: Base64VecU8(vec![]),
    };
    let dao = deploy!(
        contract: DAOContract,
        contract_id: "dao".to_string(),
        bytes: &DAO_WASM_BYTES,
        signer_account: root,
        deposit: to_yocto("200"),
        init_method: new(config, VersionedPolicy::Default(vec![root.account_id.clone()]))
    );
    (root, dao)
}

pub fn setup_test_token(root: &UserAccount) -> ContractAccount<TestTokenContract> {
    deploy!(
        contract: TestTokenContract,
        contract_id: "test_token".to_string(),
        bytes: &TEST_TOKEN_WASM_BYTES,
        signer_account: root,
        deposit: to_yocto("200"),
        init_method: new()
    )
}

pub fn setup_staking(root: &UserAccount) -> ContractAccount<StakingContract> {
    deploy!(
        contract: StakingContract,
        contract_id: "staking".to_string(),
        bytes: &STAKING_WASM_BYTES,
        signer_account: root,
        deposit: to_yocto("100"),
        init_method: new("dao".parse().unwrap(), "test_token".parse::<AccountId>().unwrap(), U64(100_000_000_000))
    )
}

pub fn add_proposal(
    root: &UserAccount,
    dao: &Contract,
    proposal: ProposalInput,
) -> ExecutionResult {
    call!(root, dao.add_proposal(proposal), deposit = to_yocto("1"))
}

pub fn add_member_proposal(
    root: &UserAccount,
    dao: &Contract,
    member_id: AccountId,
) -> ExecutionResult {
    add_proposal(
        root,
        dao,
        ProposalInput {
            description: "test".to_string(),
            kind: ProposalKind::AddMemberToRole {
                member_id: member_id,
                role: "council".to_string(),
            },
        },
    )
}

pub fn add_transfer_proposal(
    root: &UserAccount,
    dao: &Contract,
    token_id: Option<AccountId>,
    receiver_id: AccountId,
    amount: Balance,
    msg: Option<String>,
) -> ExecutionResult {
    add_proposal(
        root,
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
}

pub fn add_bounty_proposal(root: &UserAccount, dao: &Contract) -> ExecutionResult {
    add_proposal(
        root,
        dao,
        ProposalInput {
            description: "test".to_string(),
            kind: ProposalKind::AddBounty {
                bounty: Bounty {
                    description: "test bounty".to_string(),
                    token: String::from(OLD_BASE_TOKEN),
                    amount: U128(to_yocto("10")),
                    times: 3,
                    max_deadline: U64(env::block_timestamp() + 10_000_000_000),
                },
            },
        },
    )
}

pub fn vote(users: Vec<&UserAccount>, dao: &Contract, proposal_id: u64) {
    for user in users.into_iter() {
        call!(
            user,
            dao.act_proposal(proposal_id, Action::VoteApprove, None)
        )
        .assert_success();
    }
}

pub fn convert_new_to_old_token(new_account_id: Option<AccountId>) -> OldAccountId {
    if new_account_id.is_none() {
        return String::from(OLD_BASE_TOKEN);
    }
    new_account_id.unwrap().to_string()
}

 */
