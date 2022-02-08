use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap};
use near_sdk::json_types::{Base58CryptoHash, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, ext_contract, near_bindgen, sys, AccountId, Balance, BorshStorageKey, CryptoHash,
    PanicOnDefault, Promise, PromiseResult,
};

pub use crate::bounties::{Bounty, BountyClaim, VersionedBounty};
pub use crate::policy::{Policy, RoleKind, RolePermission, VersionedPolicy, VotePolicy};
use crate::proposals::VersionedProposal;
pub use crate::proposals::{Proposal, ProposalInput, ProposalKind, ProposalStatus};
pub use crate::types::{Action, Config};
use crate::upgrade::{internal_get_factory_info, internal_set_factory_info, FactoryInfo};
pub use crate::views::{BountyOutput, ProposalOutput};

mod bounties;
mod delegation;
mod policy;
mod proposals;
mod types;
mod upgrade;
pub mod views;

#[derive(BorshStorageKey, BorshSerialize)]
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

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct Contract {
    /// DAO configuration.
    pub config: LazyOption<Config>,
    /// Voting and permissions policy.
    pub policy: LazyOption<VersionedPolicy>,

    /// Amount of $NEAR locked for bonds.
    pub locked_amount: Balance,

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

#[near_bindgen]
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
            locked_amount: 0,
        };
        internal_set_factory_info(&FactoryInfo {
            factory_id: env::predecessor_account_id(),
            auto_update: true,
        });
        this
    }

    /// Should only be called by this contract on migration.
    /// This is NOOP implementation. KEEP IT if you haven't changed contract state.
    /// If you have changed state, you need to implement migration from old state (keep the old struct with different name to deserialize it first).
    /// After migrate goes live on MainNet, return this implementation for next updates.
    #[init(ignore_state)]
    pub fn migrate() -> Self {
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "ERR_NOT_ALLOWED"
        );
        let this: Contract = env::state_read().expect("ERR_CONTRACT_IS_NOT_INITIALIZED");
        this
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
        let storage_cost = ((blob_len + 32) as u128) * env::storage_byte_cost();
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
    unsafe {
        // Load input into register 0.
        sys::input(0);
        // Compute sha256 hash of register 0 and store in 1.
        sys::sha256(u64::MAX as _, 0 as _, 1);
        // Check if such blob already stored.
        assert_eq!(
            sys::storage_has_key(u64::MAX as _, 1 as _),
            0,
            "ERR_ALREADY_EXISTS"
        );
        // Get length of the input argument and check that enough $NEAR has been attached.
        let blob_len = sys::register_len(0);
        let storage_cost = ((blob_len + 32) as u128) * env::storage_byte_cost();
        assert!(
            env::attached_deposit() >= storage_cost,
            "ERR_NOT_ENOUGH_DEPOSIT:{}",
            storage_cost
        );
        // Store value of register 0 into key = register 1.
        sys::storage_write(u64::MAX as _, 1 as _, u64::MAX as _, 0 as _, 2);
        // Load register 1 into blob_hash and save into LookupMap.
        let blob_hash = [0u8; 32];
        sys::read_register(1, blob_hash.as_ptr() as _);
        contract
            .blobs
            .insert(&blob_hash, &env::predecessor_account_id());
        // Return from function value of register 1.
        let blob_hash_str = near_sdk::serde_json::to_string(&Base58CryptoHash::from(blob_hash))
            .unwrap()
            .into_bytes();
        sys::value_return(blob_hash_str.len() as _, blob_hash_str.as_ptr() as _);
    }
    env::state_write(&contract);
}

#[cfg(test)]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{testing_env, Timestamp};
    use near_sdk_sim::to_yocto;

    use crate::proposals::ProposalStatus;

    use super::*;

    fn create_proposal(
        context: &mut VMContextBuilder,
        contract: &mut Contract,
        deadline: Option<Timestamp>,
    ) -> u64 {
        testing_env!(context.attached_deposit(to_yocto("1")).build());
        contract.add_proposal(ProposalInput {
            description: "test".to_string(),
            kind: ProposalKind::Transfer {
                token_id: None,
                receiver_id: accounts(2).into(),
                amount: U128(to_yocto("100")),
                msg: None,
            },
            deadline,
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
        let id = create_proposal(&mut context, &mut contract, None);
        assert_eq!(contract.get_proposal(id).proposal.description, "test");
        assert_eq!(contract.get_proposals(0, 10).len(), 1);

        let id = create_proposal(&mut context, &mut contract, None);
        contract.act_proposal(id, Action::VoteApprove, None);
        assert_eq!(
            contract.get_proposal(id).proposal.status,
            ProposalStatus::Approved
        );

        let id = create_proposal(&mut context, &mut contract, None);
        // proposal expired, finalize.
        testing_env!(context
            .block_timestamp(1_000_000_000 * 24 * 60 * 60 * 8)
            .build());
        contract.act_proposal(id, Action::Finalize, None);
        assert_eq!(
            contract.get_proposal(id).proposal.status,
            ProposalStatus::Expired
        );

        // non council adding proposal per default policy.
        testing_env!(context
            .predecessor_account_id(accounts(2))
            .attached_deposit(to_yocto("1"))
            .build());
        let _id = contract.add_proposal(ProposalInput {
            description: "test".to_string(),
            kind: ProposalKind::AddMemberToRole {
                member_id: accounts(2).into(),
                role: "council".to_string(),
            },
            deadline: None,
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
        let id = create_proposal(&mut context, &mut contract, None);
        assert_eq!(contract.get_proposal(id).proposal.description, "test");
        contract.act_proposal(id, Action::RemoveProposal, None);
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
        let id = create_proposal(&mut context, &mut contract, None);
        assert_eq!(contract.get_proposal(id).proposal.description, "test");
        contract.act_proposal(id, Action::RemoveProposal, None);
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
        let id = create_proposal(&mut context, &mut contract, None);
        testing_env!(context
            .block_timestamp(1_000_000_000 * 24 * 60 * 60 * 8)
            .build());
        contract.act_proposal(id, Action::VoteApprove, None);
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
        let id = create_proposal(&mut context, &mut contract, None);
        contract.act_proposal(id, Action::VoteApprove, None);
        contract.act_proposal(id, Action::VoteApprove, None);
    }

    #[test]
    fn test_add_to_missing_role() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        let mut contract = Contract::new(
            Config::test_config(),
            VersionedPolicy::Default(vec![accounts(1).into()]),
        );
        testing_env!(context.attached_deposit(to_yocto("1")).build());
        let id = contract.add_proposal(ProposalInput {
            description: "test".to_string(),
            kind: ProposalKind::AddMemberToRole {
                member_id: accounts(2).into(),
                role: "missing".to_string(),
            },
            deadline: None,
        });
        contract.act_proposal(id, Action::VoteApprove, None);
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
        testing_env!(context.attached_deposit(to_yocto("1")).build());
        let _id = contract.add_proposal(ProposalInput {
            description: "test".to_string(),
            kind: ProposalKind::ChangePolicy {
                policy: VersionedPolicy::Default(vec![]),
            },
            deadline: None,
        });
    }

    pub const MIN_VOTING_TIME: Timestamp = 3 * 24 * 60 * 60 * 1_000_000_000;
    #[test]
    fn test_deadline() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        let mut policy: Policy = policy::default_policy(vec![
            accounts(1).into(),
            accounts(2).into(),
            accounts(3).into(),
            accounts(4).into(),
            accounts(5).into(),
        ]);
        policy.min_voting_time = Some(MIN_VOTING_TIME);
        let mut contract = Contract::new(Config::test_config(), VersionedPolicy::Current(policy));

        // Finalize with the only 1 approval from 5 votes AFTER the deadline
        let id = create_proposal(&mut context, &mut contract, Some(MIN_VOTING_TIME));
        contract.act_proposal(id, Action::VoteApprove, None);

        testing_env!(context.block_timestamp(MIN_VOTING_TIME).build());

        contract.act_proposal(id, Action::Finalize, None);
        assert_eq!(
            contract.get_proposal(id).proposal.status,
            ProposalStatus::Approved
        );

        // Finalize with the only 1 rejection from 5 votes AFTER the deadline
        testing_env!(context.block_timestamp(0).build());
        let id = create_proposal(&mut context, &mut contract, Some(MIN_VOTING_TIME));
        contract.act_proposal(id, Action::VoteReject, None);

        testing_env!(context.block_timestamp(MIN_VOTING_TIME).build());

        contract.act_proposal(id, Action::Finalize, None);
        assert_eq!(
            contract.get_proposal(id).proposal.status,
            ProposalStatus::Rejected
        );

        // Reject after a deadline with only 2 approvals & 1 rejection from 5 votes
        testing_env!(context
            .block_timestamp(0)
            .predecessor_account_id(accounts(1))
            .build());
        let id = create_proposal(&mut context, &mut contract, Some(MIN_VOTING_TIME));

        contract.act_proposal(id, Action::VoteApprove, None);

        testing_env!(context.predecessor_account_id(accounts(2)).build());
        contract.act_proposal(id, Action::VoteReject, None);

        testing_env!(context.predecessor_account_id(accounts(3)).build());
        contract.act_proposal(id, Action::VoteApprove, None);

        testing_env!(context
            .block_timestamp(MIN_VOTING_TIME)
            .predecessor_account_id(accounts(4))
            .build());

        contract.act_proposal(id, Action::VoteReject, None);
        assert_eq!(
            contract.get_proposal(id).proposal.status,
            ProposalStatus::Approved
        );

        // Approve after a deadline with only 1 approval & 2 rejection from 5 votes
        testing_env!(context
            .block_timestamp(0)
            .predecessor_account_id(accounts(1))
            .build());
        let id = create_proposal(&mut context, &mut contract, Some(MIN_VOTING_TIME));

        contract.act_proposal(id, Action::VoteApprove, None);

        testing_env!(context.predecessor_account_id(accounts(2)).build());
        contract.act_proposal(id, Action::VoteReject, None);

        testing_env!(context.predecessor_account_id(accounts(3)).build());
        contract.act_proposal(id, Action::VoteReject, None);

        testing_env!(context
            .block_timestamp(MIN_VOTING_TIME)
            .predecessor_account_id(accounts(4))
            .build());

        contract.act_proposal(id, Action::VoteApprove, None);

        assert_eq!(
            contract.get_proposal(id).proposal.status,
            ProposalStatus::Rejected
        );

        // Approve after a deadline with the only 1 approval from 5 votes
        testing_env!(context
            .block_timestamp(0)
            .predecessor_account_id(accounts(1))
            .build());
        let id = create_proposal(&mut context, &mut contract, Some(MIN_VOTING_TIME));
        contract.act_proposal(id, Action::VoteApprove, None);

        testing_env!(context
            .block_timestamp(MIN_VOTING_TIME)
            .predecessor_account_id(accounts(2))
            .build());

        contract.act_proposal(id, Action::VoteApprove, None);
        assert_eq!(
            contract.get_proposal(id).proposal.status,
            ProposalStatus::Approved
        );

        // Reject after a deadline with the only 1 approval from 5 votes
        testing_env!(context
            .block_timestamp(0)
            .predecessor_account_id(accounts(1))
            .build());
        let id = create_proposal(&mut context, &mut contract, Some(MIN_VOTING_TIME));
        contract.act_proposal(id, Action::VoteApprove, None);

        testing_env!(context
            .block_timestamp(MIN_VOTING_TIME)
            .predecessor_account_id(accounts(2))
            .build());

        contract.act_proposal(id, Action::VoteReject, None);
        assert_eq!(
            contract.get_proposal(id).proposal.status,
            ProposalStatus::Approved
        );

        //  Proposal with 0 votes. Finalize after deadline
        testing_env!(context.block_timestamp(0).build());
        let id = create_proposal(&mut context, &mut contract, Some(MIN_VOTING_TIME));
        testing_env!(context.block_timestamp(MIN_VOTING_TIME).build());

        contract.act_proposal(id, Action::Finalize, None);
        assert_eq!(
            contract.get_proposal(id).proposal.status,
            ProposalStatus::Removed
        );
    }

    #[test]
    #[should_panic(expected = "ERR_DEADLINE_SET_TOO_EARLY")]
    fn test_deadline_period() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        let mut policy: Policy = policy::default_policy(vec![accounts(1).into()]);
        policy.min_voting_time = Some(MIN_VOTING_TIME);
        let mut contract = Contract::new(Config::test_config(), VersionedPolicy::Current(policy));

        create_proposal(&mut context, &mut contract, Some(MIN_VOTING_TIME / 2));
    }

    #[test]
    #[should_panic(expected = "ERR_DEADLINE_FORBIDDEN_KIND")]
    fn test_deadline_type() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        let mut policy: Policy = policy::default_policy(vec![accounts(1).into()]);
        policy.min_voting_time = Some(MIN_VOTING_TIME);
        let mut contract = Contract::new(Config::test_config(), VersionedPolicy::Current(policy));

        testing_env!(context.attached_deposit(to_yocto("1")).build());
        contract.add_proposal(ProposalInput {
            description: "test".to_string(),
            kind: ProposalKind::ChangeConfig {
                config: Config::test_config(),
            },
            deadline: Some(MIN_VOTING_TIME),
        });
    }

    #[test]
    #[should_panic(expected = "ERR_MIN_VOTING_TIME_IS_MISSING")]
    fn test_deadline_without_min_voting_time() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        let mut contract = Contract::new(
            Config::test_config(),
            VersionedPolicy::Default(vec![accounts(1).into()]),
        );

        testing_env!(context.attached_deposit(to_yocto("1")).build());
        create_proposal(&mut context, &mut contract, Some(MIN_VOTING_TIME));
    }
}
