use std::cmp::min;
use std::collections::{HashMap, HashSet};

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{WrappedDuration, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, AccountId, Balance};

use crate::proposals::{Proposal, ProposalKind, ProposalStatus, Vote};
use crate::types::Action;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum RoleKind {
    /// Matches everyone, who is not matched by other roles.
    Everyone,
    /// Member greater or equal than given balance. Can use `1` as non-zero balance.
    Member(Balance),
    /// Set of accounts.
    Group(HashSet<AccountId>),
}

impl RoleKind {
    /// Checks if user matches given role.
    pub fn match_user(&self, user: &UserInfo) -> bool {
        match self {
            RoleKind::Everyone => true,
            RoleKind::Member(amount) => user.amount >= *amount,
            RoleKind::Group(accounts) => accounts.contains(&user.account_id),
        }
    }

    /// Returns the number of people in the this role or None if not supported role kind.
    pub fn get_role_size(&self) -> Option<usize> {
        match self {
            RoleKind::Group(accounts) => Some(accounts.len()),
            _ => None,
        }
    }

    pub fn add_member_to_group(&mut self, member_id: &AccountId) -> Result<(), ()> {
        match self {
            RoleKind::Group(accounts) => {
                accounts.insert(member_id.clone());
                Ok(())
            }
            _ => Err(()),
        }
    }

    pub fn remove_member_from_group(&mut self, member_id: &AccountId) -> Result<(), ()> {
        match self {
            RoleKind::Group(accounts) => {
                accounts.remove(member_id);
                Ok(())
            }
            _ => Err(()),
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct RolePermission {
    /// Name of the role to display to the user.
    pub name: String,
    /// Kind of the role: defines which users this permissions apply.
    pub kind: RoleKind,
    /// Set of actions on which proposals that this role is allowed to execute.
    /// <proposal_kind>:<action>
    pub permissions: HashSet<String>,
    /// For each proposal kind, defines voting policy.
    pub vote_policy: HashMap<String, VotePolicy>,
}

pub struct UserInfo {
    pub account_id: AccountId,
    pub amount: Balance,
}

/// Direct weight or ratio to total weight, used for the voting policy.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
#[serde(untagged)]
pub enum WeightOrRatio {
    Weight(U128),
    Ratio(u64, u64),
}

impl WeightOrRatio {
    /// Convert weight or ratio to specific weight given total weight.
    pub fn to_weight(&self, total_weight: Balance) -> Balance {
        match self {
            WeightOrRatio::Weight(weight) => min(weight.0, total_weight),
            WeightOrRatio::Ratio(num, denom) => min(
                (*num as u128 * total_weight) / *denom as u128 + 1,
                total_weight,
            ),
        }
    }
}

/// How the voting policy votes get weigthed.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum WeightKind {
    /// Using token amounts and total delegated at the moment.
    TokenWeight,
    /// Weight of the group role. Roles that don't have scoped group are not supported.
    RoleWeight,
}

/// Defines configuration of the vote.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct VotePolicy {
    /// Kind of weight to use for votes.
    pub weight_kind: WeightKind,
    /// Minimum number required for vote to finalize.
    /// If weight kind is TokenWeight - this is minimum number of tokens required.
    ///     This allows to avoid situation where the number of staked tokens from total supply is too small.
    /// If RoleWeight - this is minimum umber of votes.
    ///     This allows to avoid situation where the role is got too small but policy kept at 1/2, for example.
    pub quorum: U128,
    /// How many votes to pass this vote.
    pub threshold: WeightOrRatio,
}

impl Default for VotePolicy {
    fn default() -> Self {
        VotePolicy {
            weight_kind: WeightKind::RoleWeight,
            quorum: U128(0),
            threshold: WeightOrRatio::Ratio(1, 2),
        }
    }
}

/// Defines voting / decision making policy of this DAO.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Policy {
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

/// Versioned policy.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde", untagged)]
pub enum VersionedPolicy {
    /// Default policy with given accounts as council.
    Default(Vec<AccountId>),
    Current(Policy),
}

/// Defines default policy:
///     - everyone can add proposals
///     - group consisting of the call can do all actions, consists of caller.
///     - non token weighted voting, requires 1/2 of the group to vote
///     - proposal & bounty bond is 1N
///     - proposal & bounty forgiveness period is 1 day
fn default_policy(council: Vec<AccountId>) -> Policy {
    Policy {
        roles: vec![
            RolePermission {
                name: "all".to_string(),
                kind: RoleKind::Everyone,
                permissions: vec!["*:AddProposal".to_string()].into_iter().collect(),
                vote_policy: HashMap::default(),
            },
            RolePermission {
                name: "council".to_string(),
                kind: RoleKind::Group(council.into_iter().collect()),
                // All actions except RemoveProposal are allowed by council.
                permissions: vec![
                    "*:AddProposal".to_string(),
                    "*:VoteApprove".to_string(),
                    "*:VoteReject".to_string(),
                    "*:VoteRemove".to_string(),
                    "*:Finalize".to_string(),
                ]
                .into_iter()
                .collect(),
                vote_policy: HashMap::default(),
            },
        ],
        default_vote_policy: VotePolicy::default(),
        proposal_bond: U128(10u128.pow(24)),
        proposal_period: WrappedDuration::from(1_000_000_000 * 60 * 60 * 24 * 7),
        bounty_bond: U128(10u128.pow(24)),
        bounty_forgiveness_period: WrappedDuration::from(1_000_000_000 * 60 * 60 * 24),
    }
}

impl VersionedPolicy {
    /// Upgrades either version of policy into the latest.
    pub fn upgrade(self) -> Self {
        match self {
            VersionedPolicy::Default(accounts) => {
                VersionedPolicy::Current(default_policy(accounts))
            }
            VersionedPolicy::Current(policy) => VersionedPolicy::Current(policy),
        }
    }

    /// Return recent version of policy.
    pub fn to_policy(self) -> Policy {
        match self {
            VersionedPolicy::Current(policy) => policy,
            _ => unimplemented!(),
        }
    }

    pub fn to_policy_mut(&mut self) -> &mut Policy {
        match self {
            VersionedPolicy::Current(policy) => policy,
            _ => unimplemented!(),
        }
    }
}

impl Policy {
    ///
    /// Doesn't fail, because will be used on the finalization of the proposal.
    pub fn add_member_to_role(&mut self, role: &String, member_id: &AccountId) {
        for i in 0..self.roles.len() {
            if &self.roles[i].name == role {
                self.roles[i]
                    .kind
                    .add_member_to_group(member_id)
                    .unwrap_or_else(|()| {
                        env::log(&format!("ERR_ROLE_WRONG_KIND:{}", role).into_bytes());
                    });
                return;
            }
        }
        env::log(&format!("ERR_ROLE_NOT_FOUND:{}", role).into_bytes());
    }

    pub fn remove_member_from_role(&mut self, role: &String, member_id: &AccountId) {
        for i in 0..self.roles.len() {
            if &self.roles[i].name == role {
                self.roles[i]
                    .kind
                    .remove_member_from_group(member_id)
                    .unwrap_or_else(|()| {
                        env::log(&format!("ERR_ROLE_WRONG_KIND:{}", role).into_bytes());
                    });
                return;
            }
        }
        env::log(&format!("ERR_ROLE_NOT_FOUND:{}", role).into_bytes());
    }

    /// Returns set of roles that this user is memeber of permissions for given user across all the roles it's member of.
    fn get_user_roles(&self, user: UserInfo) -> HashMap<String, &HashSet<String>> {
        let mut roles = HashMap::default();
        for role in self.roles.iter() {
            if role.kind.match_user(&user) {
                roles.insert(role.name.clone(), &role.permissions);
            }
        }
        roles
    }

    /// Can given user execute given action on this proposal.
    /// Returns all roles that allow this action.
    pub fn can_execute_action(
        &self,
        user: UserInfo,
        proposal_kind: &ProposalKind,
        action: &Action,
    ) -> (Vec<String>, bool) {
        let roles = self.get_user_roles(user);
        let mut allowed = false;
        let allowed_roles = roles
            .into_iter()
            .filter_map(|(role, permissions)| {
                let allowed_role = permissions.contains(&format!(
                    "{}:{}",
                    proposal_kind.to_policy_label(),
                    action.to_policy_label()
                )) || permissions
                    .contains(&format!("{}:*", proposal_kind.to_policy_label()))
                    || permissions.contains(&format!("*:{}", action.to_policy_label()))
                    || permissions.contains("*:*");
                allowed = allowed || allowed_role;
                if allowed_role {
                    Some(role)
                } else {
                    None
                }
            })
            .collect();
        (allowed_roles, allowed)
    }

    /// Returns if given proposal kind is token weighted.
    pub fn is_token_weighted(&self, role: &String, proposal_kind_label: &String) -> bool {
        let role_info = self.internal_get_role(role).expect("ERR_ROLE_NOT_FOUND");
        match role_info
            .vote_policy
            .get(proposal_kind_label)
            .unwrap_or(&self.default_vote_policy)
            .weight_kind
        {
            WeightKind::TokenWeight => true,
            _ => false,
        }
    }

    fn internal_get_role(&self, name: &String) -> Option<&RolePermission> {
        for role in self.roles.iter() {
            if role.name == *name {
                return Some(role);
            }
        }
        None
    }

    /// Get proposal status for given proposal.
    /// Usually is called after changing it's state.
    pub fn proposal_status(
        &self,
        proposal: &Proposal,
        roles: Vec<String>,
        total_supply: Balance,
    ) -> ProposalStatus {
        assert_eq!(
            proposal.status,
            ProposalStatus::InProgress,
            "ERR_PROPOSAL_NOT_IN_PROGRESS"
        );
        if proposal.submission_time.0 + self.proposal_period.0 < env::block_timestamp() {
            // Proposal expired.
            return ProposalStatus::Expired;
        };
        for role in roles {
            let role_info = self.internal_get_role(&role).expect("ERR_MISSING_ROLE");
            let vote_policy = role_info
                .vote_policy
                .get(&proposal.kind.to_policy_label().to_string())
                .unwrap_or(&self.default_vote_policy);
            let threshold = std::cmp::max(
                vote_policy.quorum.0,
                match &vote_policy.weight_kind {
                    WeightKind::TokenWeight => vote_policy.threshold.to_weight(total_supply),
                    WeightKind::RoleWeight => vote_policy.threshold.to_weight(
                        role_info
                            .kind
                            .get_role_size()
                            .expect("ERR_UNSUPPORTED_ROLE") as Balance,
                    ),
                },
            );
            // Check if there is anything voted above the threshold specified by policy for given role.
            let vote_counts = proposal.vote_counts.get(&role).expect("ERR_MISSING_ROLE");
            if vote_counts[Vote::Approve as usize] >= threshold {
                return ProposalStatus::Approved;
            } else if vote_counts[Vote::Reject as usize] >= threshold {
                return ProposalStatus::Rejected;
            } else if vote_counts[Vote::Remove as usize] >= threshold {
                return ProposalStatus::Removed;
            } else {
                // continue to next role.
            }
        }
        proposal.status.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vote_policy() {
        let r1 = WeightOrRatio::Weight(U128(100));
        assert_eq!(r1.to_weight(1_000_000), 100);
        let r2 = WeightOrRatio::Ratio(1, 2);
        assert_eq!(r2.to_weight(2), 2);
        let r2 = WeightOrRatio::Ratio(1, 2);
        assert_eq!(r2.to_weight(5), 3);
        let r2 = WeightOrRatio::Ratio(1, 1);
        assert_eq!(r2.to_weight(5), 5);
    }
}
