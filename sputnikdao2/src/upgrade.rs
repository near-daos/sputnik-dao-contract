//! Logic to upgrade Sputnik contracts.

use near_sdk::borsh::to_vec;
use near_sdk::serde_json::json;
use near_sdk::{Gas, GasWeight};

use crate::*;

const FACTORY_KEY: &[u8; 7] = b"FACTORY";
const ERR_MUST_BE_SELF_OR_FACTORY: &str = "ERR_MUST_BE_SELF_OR_FACTORY";
const NO_DEPOSIT: NearToken = NearToken::from_near(0);

#[near]
#[derive(PanicOnDefault)]
pub struct ContractV1 {
    /// DAO configuration.
    pub config: LazyOption<Config>,
    /// Voting and permissions policy.
    pub policy: LazyOption<VersionedPolicy>,

    /// Amount of $NEAR locked for bonds.
    pub locked_amount: NearToken,

    /// Vote staking contract id. That contract must have this account as owner.
    pub staking_id: Option<AccountId>,
    /// Delegated  token total amount.
    pub total_delegation_amount: Balance,
    /// Delegations per user.
    pub delegations: LookupMap<AccountId, Balance>,

    /// Last available id for the proposals.
    pub last_proposal_id: u64,
    /// Proposal map from ID to proposal information.
    pub proposals: LookupMap<u64, VersionedProposal>,

    /// Last available id for the bounty.
    pub last_bounty_id: u64,
    /// Bounties map from ID to bounty information.
    pub bounties: LookupMap<u64, VersionedBounty>,
    /// Bounty claimers map per user. Allows quickly to query for each users their claims.
    pub bounty_claimers: LookupMap<AccountId, Vec<BountyClaim>>,
    /// Count of claims per bounty.
    pub bounty_claims_count: LookupMap<u64, u32>,

    /// Large blob storage.
    pub blobs: LookupMap<CryptoHash, AccountId>,
}

/// Info about factory that deployed this contract and if auto-update is allowed.
#[derive(PartialEq, Clone)]
#[near(serializers=[borsh, json])]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(deny_unknown_fields)]
pub struct FactoryInfo {
    pub factory_id: AccountId,
    pub auto_update: bool,
}

#[near]
#[derive(Debug)]
pub(crate) enum StateVersion {
    V1,
    V2,
}

const VERSION_KEY: &[u8] = b"STATEVERSION";

pub(crate) fn state_version_read() -> StateVersion {
    env::storage_read(VERSION_KEY)
        .map(|data| {
            StateVersion::try_from_slice(&data).expect("Cannot deserialize the contract state.")
        })
        .unwrap_or(StateVersion::V1)
}

pub(crate) fn state_version_write(version: &StateVersion) {
    let data = to_vec(&version).expect("Cannot serialize the contract state.");
    env::storage_write(VERSION_KEY, &data);
    near_sdk::log!("Contract state version: {:?}", version);
}

pub fn get_default_factory_id() -> AccountId {
    // ex: mydao.sputnik-dao.near
    let dao_id = env::current_account_id().to_string();
    let idx = dao_id.find('.').expect("INTERNAL_FAIL");
    // ex: sputnik-dao.near
    let factory_id = &dao_id[idx + 1..];

    factory_id.parse().unwrap()
}

/// Fetches factory info from the storage.
/// By design not using contract STATE to allow for upgrade of stuck contracts from factory.
pub(crate) fn internal_get_factory_info() -> FactoryInfo {
    env::storage_read(FACTORY_KEY)
        .map(|value| FactoryInfo::try_from_slice(&value).expect("INTERNAL_FAIL"))
        .unwrap_or_else(|| FactoryInfo {
            factory_id: get_default_factory_id(),
            auto_update: true,
        })
}

pub(crate) fn internal_set_factory_info(factory_info: &FactoryInfo) {
    let mut serialize_buf: Vec<u8> = Vec::new();
    BorshSerialize::serialize(factory_info, &mut serialize_buf).expect("INTERNAL_FAIL");
    env::storage_write(FACTORY_KEY, &serialize_buf);
}

/// Function that receives new contract, updates and calls migration.
/// Two options who call it:
///  - current account, in case of fetching contract code from factory;
///  - factory, if this contract allows to factory-update;
#[no_mangle]
pub fn update() {
    env::setup_panic_hook();

    let factory_info = internal_get_factory_info();
    let current_id = env::current_account_id();
    assert!(
        env::predecessor_account_id() == current_id
            || (env::predecessor_account_id() == factory_info.factory_id
                && factory_info.auto_update),
        "{}",
        ERR_MUST_BE_SELF_OR_FACTORY
    );

    let is_callback = env::predecessor_account_id() == current_id;
    let input;
    if is_callback {
        input = match env::promise_result(0) {
            PromiseResult::Successful(data) => data,
            _ => env::panic_str("ERR_NO_RESULT"),
        };
    } else {
        input = env::input().expect("ERR_NO_INPUT");
    };

    let promise_id = env::promise_batch_create(&current_id);
    // Deploy the contract code.
    env::promise_batch_action_deploy_contract(promise_id, &input);
    // Call promise to migrate the state.
    // Batched together to fail upgrade if migration fails.
    env::promise_batch_action_function_call_weight(
        promise_id,
        "migrate",
        &[],
        NO_DEPOSIT,
        Gas::from_gas(0),
        GasWeight::default(),
    );
    env::promise_return(promise_id);
}

pub(crate) fn upgrade_using_factory(code_hash: Base58CryptoHash) {
    let account_id = get_default_factory_id();
    // Create a promise toward the factory.
    let promise_id = env::promise_batch_create(&account_id);
    // Call `update` method from the factory which calls `update` method on this account.
    env::promise_batch_action_function_call_weight(
        promise_id,
        "update",
        &json!({ "account_id": env::current_account_id(), "code_hash": code_hash })
            .to_string()
            .into_bytes(),
        NO_DEPOSIT,
        Gas::from_gas(0),
        GasWeight::default(),
    );
    env::promise_return(promise_id);
}

#[allow(dead_code)]
pub(crate) fn upgrade_self(hash: &[u8]) {
    let current_id = env::current_account_id();
    let input = env::storage_read(hash).expect("ERR_NO_HASH");
    let promise_id = env::promise_batch_create(&current_id);
    env::promise_batch_action_deploy_contract(promise_id, &input);
    env::promise_batch_action_function_call_weight(
        promise_id,
        "migrate",
        &[],
        NO_DEPOSIT,
        Gas::from_gas(0),
        GasWeight::default(),
    );
}

pub(crate) fn upgrade_remote(receiver_id: &AccountId, method_name: &str, hash: &[u8]) {
    let input = env::storage_read(hash).expect("ERR_NO_HASH");
    let promise_id = env::promise_batch_create(receiver_id);

    env::promise_batch_action_function_call_weight(
        promise_id,
        method_name,
        &input,
        NO_DEPOSIT,
        Gas::from_gas(0),
        GasWeight::default(),
    );
}
