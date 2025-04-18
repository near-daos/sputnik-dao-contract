use std::collections::VecDeque;

use near_contract_standards::fungible_token::Balance;
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap};
use near_sdk::json_types::{Base58CryptoHash, U128};
use near_sdk::{
    env, ext_contract, near, AccountId, BorshStorageKey, CryptoHash, NearToken, PanicOnDefault,
    Promise, PromiseOrValue, PromiseResult,
};

use crate::action_log::ActionLog;
pub use crate::bounties::{Bounty, BountyClaim, VersionedBounty};
pub use crate::policy::{
    default_policy, Policy, RoleKind, RolePermission, VersionedPolicy, VotePolicy,
};
use crate::proposals::VersionedProposal;
pub use crate::proposals::{Proposal, ProposalInput, ProposalKind, ProposalStatus};
pub use crate::types::{Action, Config, OldAccountId, OLD_BASE_TOKEN};
use crate::upgrade::{
    internal_get_factory_info, internal_set_factory_info, state_version_read, state_version_write,
    ContractV1, FactoryInfo, StateVersion,
};
pub use crate::views::{BountyOutput, ProposalOutput};

pub mod action_log;
mod bounties;
mod delegation;
mod ext_fungible_token;
mod policy;
pub mod proposals;
mod types;
mod upgrade;
pub mod views;

#[near(serializers=[borsh])]
#[derive(BorshStorageKey)]
pub enum StorageKeys {
    Config,
    Policy,
    Delegations,
    Proposals,
    Bounties,
    BountyClaimers,
    BountyClaimCounts,
    Blobs,
}

/// After payouts, allows a callback
#[ext_contract(ext_self)]
pub trait ExtSelf {
    /// Callback after proposal execution.
    fn on_proposal_callback(&mut self, proposal_id: u64) -> PromiseOrValue<()>;
}

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct Contract {
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

    /// Log of the latest actions on proposals
    pub actions_log: VecDeque<ActionLog>,
}

#[near]
impl Contract {
    #[init]
    pub fn new(config: Config, policy: VersionedPolicy) -> Self {
        let this = Self {
            config: LazyOption::new(StorageKeys::Config, Some(&config)),
            policy: LazyOption::new(StorageKeys::Policy, Some(&policy.upgrade())),
            staking_id: None,
            total_delegation_amount: 0,
            delegations: LookupMap::new(StorageKeys::Delegations),
            last_proposal_id: 0,
            proposals: LookupMap::new(StorageKeys::Proposals),
            last_bounty_id: 0,
            bounties: LookupMap::new(StorageKeys::Bounties),
            bounty_claimers: LookupMap::new(StorageKeys::BountyClaimers),
            bounty_claims_count: LookupMap::new(StorageKeys::BountyClaimCounts),
            blobs: LookupMap::new(StorageKeys::Blobs),
            locked_amount: NearToken::from_near(0),
            actions_log: VecDeque::new(),
        };
        internal_set_factory_info(&FactoryInfo {
            factory_id: env::predecessor_account_id(),
            auto_update: true,
        });
        state_version_write(&StateVersion::V2);
        this
    }

    /// Should only be called by this contract on migration.
    /// This is NOOP implementation. KEEP IT if you haven't changed contract state.
    /// If you have changed state, you need to implement migration from old state (keep the old struct with different name to deserialize it first).
    /// After migrate goes live on MainNet, return this implementation for next updates.
    #[private]
    #[init(ignore_state)]
    pub fn migrate() -> Self {
        let version = state_version_read();
        match version {
            StateVersion::V1 => {
                let this: ContractV1 = env::state_read().expect("ERR_CONTRACT_IS_NOT_INITIALIZED");
                state_version_write(&StateVersion::V2);
                Contract {
                    config: this.config,
                    policy: this.policy,
                    locked_amount: this.locked_amount,
                    staking_id: this.staking_id,
                    total_delegation_amount: this.total_delegation_amount,
                    delegations: this.delegations,
                    last_proposal_id: this.last_proposal_id,
                    proposals: this.proposals,
                    last_bounty_id: this.last_bounty_id,
                    bounties: this.bounties,
                    bounty_claimers: this.bounty_claimers,
                    bounty_claims_count: this.bounty_claims_count,
                    blobs: this.blobs,
                    actions_log: VecDeque::new(),
                }
            }
            StateVersion::V2 => env::state_read().expect("ERR_CONTRACT_IS_NOT_INITIALIZED"),
        }
    }

    /// Remove blob from contract storage and pay back to original storer.
    /// Only original storer can call this.
    pub fn remove_blob(&mut self, hash: Base58CryptoHash) -> Promise {
        let hash: CryptoHash = hash.into();
        let account_id = self.blobs.remove(&hash).expect("ERR_NO_BLOB");
        assert_eq!(
            env::predecessor_account_id(),
            account_id,
            "ERR_INVALID_CALLER"
        );
        env::storage_remove(&hash);
        let blob_len = env::register_len(u64::MAX - 1).unwrap();
        let storage_cost = env::storage_byte_cost().saturating_mul((blob_len + 32) as u128);
        Promise::new(account_id).transfer(storage_cost)
    }

    /// Returns factory information, including if auto update is allowed.
    pub fn get_factory_info(&self) -> FactoryInfo {
        internal_get_factory_info()
    }
}

/// Stores attached data into blob store and returns hash of it.
/// Implemented to avoid loading the data into WASM for optimal gas usage.
#[no_mangle]
pub extern "C" fn store_blob() {
    env::setup_panic_hook();
    let mut contract: Contract = env::state_read().expect("ERR_CONTRACT_IS_NOT_INITIALIZED");
    let input = env::input().expect("ERR_NO_INPUT");
    let sha256_hash = env::sha256(&input);
    assert!(!env::storage_has_key(&sha256_hash), "ERR_ALREADY_EXISTS");

    let blob_len = input.len();
    let storage_cost = env::storage_byte_cost().saturating_mul((blob_len + 32) as u128);
    assert!(
        env::attached_deposit() >= storage_cost,
        "ERR_NOT_ENOUGH_DEPOSIT:{}",
        storage_cost
    );

    env::storage_write(&sha256_hash, &input);
    let mut blob_hash = [0u8; 32];
    blob_hash.copy_from_slice(&sha256_hash);
    contract
        .blobs
        .insert(&blob_hash, &env::predecessor_account_id());
    let blob_hash_str = near_sdk::serde_json::to_string(&Base58CryptoHash::from(blob_hash))
        .unwrap()
        .into_bytes();

    env::value_return(&blob_hash_str);
    env::state_write(&contract);
}

#[cfg(test)]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;
    use near_workspaces::types::NearToken;

    use crate::action_log::ProposalLog;
    use crate::proposals::ProposalStatus;

    use super::*;

    fn create_proposal(context: &mut VMContextBuilder, contract: &mut Contract) -> u64 {
        testing_env!(context.attached_deposit(NearToken::from_near(1)).build());
        contract.add_proposal(ProposalInput {
            description: "test".to_string(),
            kind: ProposalKind::Transfer {
                token_id: String::from(OLD_BASE_TOKEN),
                receiver_id: accounts(2).into(),
                amount: U128(NearToken::from_near(100).as_yoctonear()),
                msg: None,
            },
        })
    }

    #[test]
    fn test_basics() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        let mut contract = Contract::new(
            Config::test_config(),
            VersionedPolicy::Default(vec![accounts(1).into()]),
        );
        let id = create_proposal(&mut context, &mut contract);
        assert_eq!(contract.get_proposal(id).proposal.description, "test");
        assert_eq!(contract.get_proposals(0, 10).len(), 1);

        let id = create_proposal(&mut context, &mut contract);
        contract.act_proposal(
            id,
            Action::VoteApprove,
            contract.get_proposal(id).proposal.kind,
            None,
        );
        assert_eq!(
            contract.get_proposal(id).proposal.status,
            ProposalStatus::Approved
        );

        let id = create_proposal(&mut context, &mut contract);
        // proposal expired, finalize.
        testing_env!(context
            .block_timestamp(1_000_000_000 * 24 * 60 * 60 * 8)
            .build());
        contract.act_proposal(
            id,
            Action::Finalize,
            contract.get_proposal(id).proposal.kind,
            None,
        );
        assert_eq!(
            contract.get_proposal(id).proposal.status,
            ProposalStatus::Expired
        );

        // non council adding proposal per default policy.
        testing_env!(context
            .predecessor_account_id(accounts(2))
            .attached_deposit(NearToken::from_near(1))
            .build());
        let _id = contract.add_proposal(ProposalInput {
            description: "test".to_string(),
            kind: ProposalKind::AddMemberToRole {
                member_id: accounts(2).into(),
                role: "council".to_string(),
            },
        });
    }

    #[test]
    #[should_panic(expected = "ERR_PERMISSION_DENIED")]
    fn test_remove_proposal_denied() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        let mut contract = Contract::new(
            Config::test_config(),
            VersionedPolicy::Default(vec![accounts(1).into()]),
        );
        let id = create_proposal(&mut context, &mut contract);
        assert_eq!(contract.get_proposal(id).proposal.description, "test");
        contract.act_proposal(
            id,
            Action::RemoveProposal,
            contract.get_proposal(id).proposal.kind,
            None,
        );
    }

    #[test]
    fn test_remove_proposal_allowed() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        let mut policy = VersionedPolicy::Default(vec![accounts(1).into()]).upgrade();
        policy.to_policy_mut().roles[1]
            .permissions
            .insert("*:RemoveProposal".to_string());
        let mut contract = Contract::new(Config::test_config(), policy);
        let id = create_proposal(&mut context, &mut contract);
        assert_eq!(contract.get_proposal(id).proposal.description, "test");
        contract.act_proposal(
            id,
            Action::RemoveProposal,
            contract.get_proposal(id).proposal.kind,
            None,
        );
        assert_eq!(contract.get_proposals(0, 10).len(), 0);
    }

    #[test]
    fn test_vote_expired_proposal() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        let mut contract = Contract::new(
            Config::test_config(),
            VersionedPolicy::Default(vec![accounts(1).into()]),
        );
        let id = create_proposal(&mut context, &mut contract);
        testing_env!(context
            .block_timestamp(1_000_000_000 * 24 * 60 * 60 * 8)
            .build());
        contract.act_proposal(
            id,
            Action::VoteApprove,
            contract.get_proposal(id).proposal.kind,
            None,
        );
    }

    #[test]
    #[should_panic(expected = "ERR_ALREADY_VOTED")]
    fn test_vote_twice() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        let mut contract = Contract::new(
            Config::test_config(),
            VersionedPolicy::Default(vec![accounts(1).into(), accounts(2).into()]),
        );
        let id = create_proposal(&mut context, &mut contract);
        contract.act_proposal(
            id,
            Action::VoteApprove,
            contract.get_proposal(id).proposal.kind,
            None,
        );
        contract.act_proposal(
            id,
            Action::VoteApprove,
            contract.get_proposal(id).proposal.kind,
            None,
        );
    }

    #[test]
    #[should_panic(expected = "ERR_WRONG_KIND")]
    fn test_wrong_kind() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        let mut contract = Contract::new(
            Config::test_config(),
            VersionedPolicy::Default(vec![accounts(1).into(), accounts(2).into()]),
        );
        let id = create_proposal(&mut context, &mut contract);
        contract.act_proposal(
            id,
            Action::VoteApprove,
            ProposalKind::Transfer {
                token_id: String::from(OLD_BASE_TOKEN),
                receiver_id: accounts(1).into(), // The only different thing from initial kind
                amount: U128(NearToken::from_near(100).as_yoctonear()),
                msg: None,
            },
            None,
        );
    }

    #[test]
    fn test_add_to_missing_role() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        let mut contract = Contract::new(
            Config::test_config(),
            VersionedPolicy::Default(vec![accounts(1).into()]),
        );
        testing_env!(context.attached_deposit(NearToken::from_near(1)).build());
        let id = contract.add_proposal(ProposalInput {
            description: "test".to_string(),
            kind: ProposalKind::AddMemberToRole {
                member_id: accounts(2).into(),
                role: "missing".to_string(),
            },
        });
        contract.act_proposal(
            id,
            Action::VoteApprove,
            contract.get_proposal(id).proposal.kind,
            None,
        );
        let x = contract.get_policy();
        // still 2 roles: all and council.
        assert_eq!(x.roles.len(), 2);
    }

    #[test]
    #[should_panic(expected = "ERR_INVALID_POLICY")]
    fn test_fails_adding_invalid_policy() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        let mut contract = Contract::new(
            Config::test_config(),
            VersionedPolicy::Default(vec![accounts(1).into()]),
        );
        testing_env!(context.attached_deposit(NearToken::from_near(1)).build());
        let _id = contract.add_proposal(ProposalInput {
            description: "test".to_string(),
            kind: ProposalKind::ChangePolicy {
                policy: VersionedPolicy::Default(vec![]),
            },
        });
    }

    #[test]
    fn test_action_log() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.predecessor_account_id(accounts(0)).build());
        let accounts_list: Vec<AccountId> = [
            "alice", "bob", "charlie", "danny", "eugene", "fargo", "grace", "hannah", "ian",
            "julia", "kevin", "linda", "michael", "nathan", "olivia", "patricia", "quinn",
            "robert", "sarah", "thomas", "ursula", "victor", "wendy", "xavier", "yasmine", "zack",
            "adam", "bella", "cameron", "diana", "ethan", "fiona", "george", "heidi", "isaac",
            "jennifer", "kyle", "lily", "marcus", "nina", "oscar", "penelope", "quincy", "ryan",
            "sophia", "tyler", "uma", "vincent", "willow", "xander",
        ]
        .iter()
        .map(|name| name.parse().unwrap())
        .collect();
        let mut contract = Contract::new(
            Config::test_config(),
            VersionedPolicy::Default(accounts_list.clone()),
        );
        let id = create_proposal(&mut context, &mut contract);

        // Check that last actions log has proposal create
        let proposal = contract.get_proposal(id).proposal;
        let proposal_kind = contract.get_proposal(id).proposal.kind;
        let proposal_last_actions_log = proposal.last_actions_log;
        let global_last_actions_log = contract.get_actions_log();
        assert_eq!(proposal_last_actions_log.len(), 1);
        assert_eq!(global_last_actions_log.len(), 1);
        assert_eq!(
            proposal_last_actions_log[0],
            ProposalLog {
                block_height: 0.into()
            }
        );
        assert_eq!(
            global_last_actions_log.get(0).unwrap().clone(),
            ActionLog {
                account_id: "alice".parse().unwrap(),
                proposal_id: 0.into(),
                action: Action::AddProposal,
                block_height: 0.into()
            }
        );

        // Fill the latest actions list
        for i in 1..21 {
            testing_env!(context
                .predecessor_account_id(accounts_list.get(i).unwrap().clone())
                .build());
            contract.act_proposal(id, Action::VoteApprove, proposal_kind.clone(), None);
        }
        // Now the oldest proposal should be bob's vote
        let proposal_last_actions_log = contract.get_proposal(id).proposal.last_actions_log;
        let global_last_actions_log = contract.get_actions_log();
        assert_eq!(proposal_last_actions_log.len(), 20);
        assert_eq!(global_last_actions_log.len(), 20);
        assert_eq!(
            proposal_last_actions_log[0],
            ProposalLog {
                block_height: 0.into()
            }
        );
        assert_eq!(
            global_last_actions_log[0],
            ActionLog {
                account_id: "bob".parse().unwrap(),
                proposal_id: 0.into(),
                action: Action::VoteApprove,
                block_height: 0.into()
            }
        );
    }
}
