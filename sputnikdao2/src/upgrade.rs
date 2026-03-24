//! Logic to upgrade Sputnik contracts.

use near_sdk::borsh::to_vec;
use near_sdk::json_types::{Base58CryptoHash, U64};
use near_sdk::serde_json::json;
use near_sdk::{Gas, GasWeight, PromiseResult};

use crate::proposals::VersionedProposal;
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
            StateVersion::try_from_slice(&data)
                .expect("Cannot deserialize the contract state version.")
        })
        .unwrap_or(StateVersion::V1)
}

pub(crate) fn state_version_write(version: &StateVersion) {
    let data = to_vec(&version).expect("Cannot serialize the contract state version.");
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

/// Two weeks in nanoseconds — the minimum delay before an auto-update proposal can be executed.
const TWO_WEEKS_NS: u64 = 14 * 24 * 60 * 60 * 1_000_000_000;

/// Function that receives new contract, updates and calls migration.
/// Two options who call it:
///  - current account, in case of fetching contract code from factory (self-callback);
///  - factory, if this contract allows to factory-update (auto-update);
///
/// For auto-update requests from the factory the flow is:
///  1. Check the last 5 proposals for an InProgress `UpgradeSelf` proposal whose hash
///     matches the incoming code.
///  2. If such a proposal exists **and** its `submission_time` is older than 2 weeks,
///     mark it Approved and proceed with deploying the code.
///  3. If the proposal exists but is younger than 2 weeks, do nothing (wait).
///  4. If no matching proposal exists, create a new `UpgradeSelf` proposal and return
///     without deploying.
// NOTE: Remove the #[cfg] after https://github.com/near/cargo-near/issues/317 is resolved.
#[cfg_attr(target_arch = "wasm32", unsafe(no_mangle))]
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
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
    // While it would be more efficient to pass contract code hash instead of code itself,
    // we need to support old versions of the contract that expect the code itself.
    let new_contract_code = if is_callback {
        // NOTE: It is fine to use the unbounded promise_result since the callback is always
        // triggered by a trusted party (the contract itself or from the factory).
        #[allow(deprecated)]
        let PromiseResult::Successful(data) = env::promise_result(0) else {
            env::panic_str("ERR_NO_RESULT");
        };
        data
    } else {
        env::input().expect("ERR_NO_INPUT")
    };
    let new_contract_code_hash = env::sha256_array(&new_contract_code);

    // --- Auto-update proposal gate ---
    // When the factory triggers an auto-update (not a self-callback), enforce a 2-week
    // proposal delay before the code is actually deployed, unless the contract state is broken.
    let is_auto_update = !is_callback && env::predecessor_account_id() == factory_info.factory_id;

    let contract: Option<Contract> = env::state_read();

    if let (Some(mut contract), true) = (contract, is_auto_update) {
        // Scan the last 5 proposals for a matching InProgress UpgradeSelf proposal.
        let last_id = contract.last_proposal_id;
        let start_id = last_id.saturating_sub(5);
        let mut found: Option<(u64, Proposal)> = None;

        for id in start_id..last_id {
            if let Some(versioned) = contract.proposals.get(&id) {
                let proposal: Proposal = versioned.into();
                if matches!(
                    &proposal.kind,
                    ProposalKind::UpgradeSelf { hash } if *hash == new_contract_code_hash
                ) && proposal.status == ProposalStatus::InProgress
                {
                    found = Some((id, proposal));
                    break;
                }
            }
        }

        match found {
            Some((id, mut proposal)) => {
                if env::block_timestamp().saturating_sub(proposal.submission_time.0) >= TWO_WEEKS_NS
                {
                    // Proposal is old enough — mark it as Approved and proceed
                    // with the deployment below.
                    proposal.status = ProposalStatus::Approved;
                    contract
                        .proposals
                        .insert(&id, &VersionedProposal::Latest(proposal));
                    env::state_write(&contract);
                    // Fall through to deploy the code.
                } else {
                    // Proposal exists but the 2-week waiting period has not elapsed yet.
                    return;
                }
            }
            None => {
                // No matching proposal found — create one and return without deploying.
                let proposal = Proposal {
                    proposer: factory_info.factory_id.clone(),
                    description: "Auto-update from factory".to_string(),
                    kind: ProposalKind::UpgradeSelf {
                        hash: new_contract_code_hash.into(),
                    },
                    status: ProposalStatus::InProgress,
                    vote_counts: Default::default(),
                    votes: Default::default(),
                    submission_time: U64::from(env::block_timestamp()),
                    last_actions_log: Default::default(),
                };
                let id = contract.last_proposal_id;
                contract
                    .proposals
                    .insert(&id, &VersionedProposal::Latest(proposal));
                contract.last_proposal_id += 1;
                env::state_write(&contract);
                return;
            }
        }
    };

    let promise_id = env::promise_batch_create(&current_id);
    // Deploy the contract code.
    env::promise_batch_action_use_global_contract(promise_id, &new_contract_code_hash);
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

pub(crate) fn upgrade_using_factory(code_hash: &Base58CryptoHash) {
    let factory_account_id = get_default_factory_id();
    // Create a promise toward the factory.
    let promise_id = env::promise_batch_create(&factory_account_id);
    // Call `update` method from the factory which calls `update` method on this account.
    env::promise_batch_action_function_call_weight(
        promise_id,
        "update",
        json!({ "account_id": env::current_account_id(), "code_hash": code_hash })
            .to_string()
            .as_bytes(),
        NO_DEPOSIT,
        Gas::from_gas(0),
        GasWeight(1),
    );
    env::promise_return(promise_id);
}

pub(crate) fn upgrade_remote(receiver_id: &AccountId, method_name: &str, hash: &CryptoHash) {
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
