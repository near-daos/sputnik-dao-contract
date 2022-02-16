/// This file contains old structs that are required for the v2 -> v3 migration.
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap};
use near_sdk::json_types::{U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, Balance, CryptoHash, PanicOnDefault};

pub use crate::bounties::{BountyClaim, VersionedBounty};
pub use crate::policy::{Policy, RoleKind, RolePermission, VersionedPolicy, VotePolicy};
pub use crate::proposals::{
    Proposal, ProposalInput, ProposalKind, ProposalStatus, VersionedProposal,
};
pub use crate::types::{Action, Config};

type WrappedDuration = U64;
type WrappedTimestamp = U64;

#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct OldContract {
    /// DAO configuration.
    pub config: LazyOption<Config>,
    /// Voting and permissions policy.
    pub policy: LazyOption<OldVersionedPolicy>,

    /// Amount of $NEAR locked for storage / bonds.
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
    pub bounties: LookupMap<u64, OldVersionedBounty>,
    /// Bounty claimers map per user. Allows quickly to query for each users their claims.
    pub bounty_claimers: LookupMap<AccountId, Vec<OldBountyClaim>>,
    /// Count of claims per bounty.
    pub bounty_claims_count: LookupMap<u64, u32>,

    /// Large blob storage.
    pub blobs: LookupMap<CryptoHash, AccountId>,
}

/// Versioned policy.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde", untagged)]
pub enum OldVersionedPolicy {
    /// Default policy with given accounts as council.
    Default(Vec<AccountId>),
    Current(OldPolicy),
}

/// Defines voting / decision making policy of this DAO.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct OldPolicy {
    /// List of roles and permissions for them in the current policy.
    pub roles: Vec<RolePermission>,
    /// Default vote policy. Used when given proposal kind doesn't have special policy.
    pub default_vote_policy: VotePolicy,
    /// Proposal bond.
    pub proposal_bond: U128,
    /// Expiration period for proposals.
    pub proposal_period: WrappedDuration,
    /// Bond for claiming a bounty.
    pub bounty_bond: U128,
    /// Period in which giving up on bounty is not punished.
    pub bounty_forgiveness_period: WrappedDuration,
}

/// Information recorded about claim of the bounty by given user.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct OldBountyClaim {
    /// Bounty id that was claimed.
    bounty_id: u64,
    /// Start time of the claim.
    start_time: WrappedTimestamp,
    /// Deadline specified by claimer.
    deadline: WrappedDuration,
    /// Completed?
    completed: bool,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum OldVersionedBounty {
    Default(OldBounty),
}

/// Bounty information.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct OldBounty {
    /// Description of the bounty.
    pub description: String,
    /// Token the bounty will be paid out.
    pub token: AccountId,
    /// Amount to be paid out.
    pub amount: U128,
    /// How many times this bounty can be done.
    pub times: u32,
    /// Max deadline from claim that can be spend on this bounty.
    pub max_deadline: WrappedDuration,
}
