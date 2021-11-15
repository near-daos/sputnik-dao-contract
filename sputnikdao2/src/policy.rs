use std::cmp::min;
use std::collections::{HashMap, HashSet};

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, AccountId, Balance};

use crate::proposals::{Proposal, ProposalKind, ProposalStatus, Vote};
use crate::types::Action;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub enum Members {
    /// Matches everyone, who is not matched by other councils.
    Everyone,
    /// Member greater or equal than given balance. Can use `1` as non-zero balance.
    MinimumBalance(U128),
    /// Set of accounts.
    Group(HashSet<AccountId>),
}

impl Members {
    /// Checks if user matches given council.
    pub fn match_user(&self, user: &UserInfo) -> bool {
        match self {
            Members::Everyone => true,
            Members::MinimumBalance(amount) => user.amount >= amount.0,
            Members::Group(accounts) => accounts.contains(&user.account_id),
        }
    }

    /// Returns the number of people in the this council or None if not supported council kind.
    pub fn get_council_size(&self) -> Option<usize> {
        match self {
            Members::Group(accounts) => Some(accounts.len()),
            _ => None,
        }
    }

    pub fn add_member_to_group(&mut self, member_id: &AccountId) -> Result<(), ()> {
        match self {
            Members::Group(accounts) => {
                accounts.insert(member_id.clone());
                Ok(())
            }
            _ => Err(()),
        }
    }

    pub fn remove_member_from_group(&mut self, member_id: &AccountId) -> Result<(), ()> {
        match self {
            Members::Group(accounts) => {
                accounts.remove(member_id);
                Ok(())
            }
            _ => Err(()),
        }
    }
}

/// Councils are entities that can make a decision on the state of a
/// proposal, for every step of the proposals lifecycle.  
/// Whether proposals are created, get approved, get rejected, and so on.  
/// Although users can "act" on proposals, this should be
/// understood as making "action intentions" on proposals, since ultimately
/// only Councils decide what a proposal's state will be.
///
/// The first Council that is able to make a decision on a proposal sets
/// that proposal's state accordingly to it's decision.
///
/// There can be multiple Councils in the DAO, and each has it's own rules
/// on how they can make a decision, and their own limitations on what
/// kind of proposals they can decide/act upon.
///
/// When a user acts on a proposal, eg. trying to approve some proposal,
/// first their intention must be allowed to be registered in the system.
/// If it's not registered in the system, it's effectively ignored and
/// won't be visible to the Councils, and thus will have no effect on the
/// proposal's state.
///
/// For a user's _action_ on _a proposal_ to be registered in the system,
/// that user must be a member of a Council which is also related
/// to both that _kind of action_ and that _kind of proposal._  
/// See [`Council::members`] and [`Council::permissions`] for more info.
///
/// From the perspective of the Councils, when a user acts on a proposal,
/// it's as if all Councils try to make a decision, since there is a new
/// information in the system and no Council was able to make a decision on
/// that proposal so far. Then if any of them is able to, that decision
/// sets the proposal's state.  
/// (note: only the Councils that has that user as it's member actually
/// once more tries to make a decision).
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
#[serde(crate = "near_sdk::serde")]
pub struct Council {
    /// Name of the Council to display to the users.
    pub name: CouncilName,
    /// Members of the Council.
    /// Defines the users whose actions the Council will observe and base
    /// itself upon when trying to make a decision.
    ///
    /// If a user is trying to act on a proposal but they are not a
    /// member of any Council that gives relevance to that kind of
    /// proposal (and to that kind of action), then that user's action is
    /// denied/ignored by the system.
    pub members: Members,
    /// Set of proposal actions (on certain kinds of proposals) that this
    /// Council is able to execute.
    ///
    /// As hinted in the [`Council::members`] field, this effectivelly sets
    /// which kind of actions the `members` are _allowed_ to make, on
    /// which kind of proposals.
    ///
    /// See [`Permission`] for more information.
    pub permissions: HashSet<Permission>,
    /// For the Council when making a decision on a proposal of some kind,
    /// use a certain `VotePolicy` to analyze the members' votes.
    pub vote_policy: HashMap<ProposalKindLabel, VotePolicy>,
}

pub type CouncilName = String;

/// A proposal action (on a kind of proposal) that a [`Council`]
/// is able to execute.
///
/// The value is stringfied as:  
/// <proposal_kind>:<proposal_action>  
/// Where those values are given by [`ProposalKind::to_policy_label()`]
/// and [`Action::to_policy_label()`] respectively, and each value can
/// be a `*` to represent "any" variant.
///
/// Example 1: `"*:AddProposal` means that when the Council's
/// members indicate that they want to create a new proposal,
/// which can be any kind of proposal (`*`), the Council is able to
/// decide that creation should happen.  
/// Adding a proposal is a special proposal action because it's the
/// first step in a proposal's lifecycle. In this case the Council
/// decides immediately without depending on analyzing any votes.  
/// If the Council's members is `Everyone`, then anyone is able
/// to create any kind of proposals.
///
/// Example 2: `"ChangePolicy:VoteReject"` means that when the
/// Council's members indicate that they want to vote in rejection to a
/// specific proposal (that is trying to change the DAO's policy), the
/// Council will consider and base itself on their votes when trying to
/// make a decision. If, according to [`Council::vote_policy`], the
/// Council is able to make a decision to "reject" the proposal, then
/// that `ChangePolicy` proposal will have it's state set as
/// "rejected".  
///
/// See also: [`Council::permissions`].
pub type Permission = String;

/// The label of some kind of proposal.  
/// The value is given by [`ProposalKind::to_policy_label()`].
///
/// See also: [`Council::vote_policy`].
pub type ProposalKindLabel = String;

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
    /// Weight of the group council. Councils that don't have scoped group are not supported.
    CouncilWeight,
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
    /// If CouncilWeight - this is minimum umber of votes.
    ///     This allows to avoid situation where the council is got too small but policy kept at 1/2, for example.
    pub quorum: U128,
    /// How many votes to pass this vote.
    pub threshold: WeightOrRatio,
}

impl Default for VotePolicy {
    fn default() -> Self {
        VotePolicy {
            weight_kind: WeightKind::CouncilWeight,
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
    /// List of councils and permissions for them in the current policy.
    pub councils: Vec<Council>,
    /// Default vote policy. Used when given proposal kind doesn't have special policy.
    pub default_vote_policy: VotePolicy,
    /// Proposal bond.
    pub proposal_bond: U128,
    /// Expiration period for proposals.
    pub proposal_period: U64,
    /// Bond for claiming a bounty.
    pub bounty_bond: U128,
    /// Period in which giving up on bounty is not punished.
    pub bounty_forgiveness_period: U64,
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
        councils: vec![
            Council {
                name: "all".to_string(),
                members: Members::Everyone,
                permissions: vec!["*:AddProposal".to_string()].into_iter().collect(),
                vote_policy: HashMap::default(),
            },
            Council {
                name: "council".to_string(),
                members: Members::Group(council.into_iter().collect()),
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
        proposal_period: U64::from(1_000_000_000 * 60 * 60 * 24 * 7),
        bounty_bond: U128(10u128.pow(24)),
        bounty_forgiveness_period: U64::from(1_000_000_000 * 60 * 60 * 24),
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
    pub fn add_member_to_council(&mut self, council_name: &CouncilName, member_id: &AccountId) {
        for i in 0..self.councils.len() {
            if &self.councils[i].name == council_name {
                self.councils[i]
                    .members
                    .add_member_to_group(member_id)
                    .unwrap_or_else(|()| {
                        env::log_str(&format!("ERR_COUNCIL_WRONG_MEMBER:{}", council_name));
                    });
                return;
            }
        }
        env::log_str(&format!("ERR_COUNCIL_NOT_FOUND:{}", council_name));
    }

    pub fn remove_member_from_council(
        &mut self,
        council_name: &CouncilName,
        member_id: &AccountId,
    ) {
        for i in 0..self.councils.len() {
            if &self.councils[i].name == council_name {
                self.councils[i]
                    .members
                    .remove_member_from_group(member_id)
                    .unwrap_or_else(|()| {
                        env::log_str(&format!("ERR_COUNCIL_WRONG_MEMBER:{}", council_name));
                    });
                return;
            }
        }
        env::log_str(&format!("ERR_COUNCIL_NOT_FOUND:{}", council_name));
    }

    /// Returns set of councils that this user is memeber of permissions for given user across all the councils it's member of.
    fn get_user_councils(&self, user: UserInfo) -> HashMap<CouncilName, &HashSet<Permission>> {
        let mut councils = HashMap::default();
        for council in self.councils.iter() {
            if council.members.match_user(&user) {
                councils.insert(council.name.clone(), &council.permissions);
            }
        }
        councils
    }

    /// Can given user execute given action on this proposal.
    /// Returns all councils that allow this action.
    pub fn can_execute_action(
        &self,
        user: UserInfo,
        proposal_kind: &ProposalKind,
        action: &Action,
    ) -> (Vec<CouncilName>, bool) {
        let councils = self.get_user_councils(user);
        let mut allowed = false;
        let allowed_councils = councils
            .into_iter()
            .filter_map(|(council_name, permissions)| {
                let allowed_council = permissions.contains(&format!(
                    "{}:{}",
                    proposal_kind.to_policy_label(),
                    action.to_policy_label()
                )) || permissions
                    .contains(&format!("{}:*", proposal_kind.to_policy_label()))
                    || permissions.contains(&format!("*:{}", action.to_policy_label()))
                    || permissions.contains("*:*");
                allowed = allowed || allowed_council;
                if allowed_council {
                    Some(council_name)
                } else {
                    None
                }
            })
            .collect();
        (allowed_councils, allowed)
    }

    /// Returns if given proposal kind is token weighted.
    pub fn is_token_weighted(
        &self,
        council_name: &CouncilName,
        proposal_kind_label: &ProposalKindLabel,
    ) -> bool {
        let council_info = self
            .internal_get_council(council_name)
            .expect("ERR_COUNCIL_NOT_FOUND");
        match council_info
            .vote_policy
            .get(proposal_kind_label)
            .unwrap_or(&self.default_vote_policy)
            .weight_kind
        {
            WeightKind::TokenWeight => true,
            _ => false,
        }
    }

    fn internal_get_council(&self, name: &CouncilName) -> Option<&Council> {
        for council in self.councils.iter() {
            if council.name == *name {
                return Some(council);
            }
        }
        None
    }

    /// Get proposal status for given proposal.
    /// Usually is called after changing it's state.
    pub fn proposal_status(
        &self,
        proposal: &Proposal,
        council_names: Vec<CouncilName>,
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
        for council_name in council_names {
            let council_info = self
                .internal_get_council(&council_name)
                .expect("ERR_MISSING_COUNCIL");
            let vote_policy = council_info
                .vote_policy
                .get(&proposal.kind.to_policy_label().to_string())
                .unwrap_or(&self.default_vote_policy);
            let threshold = std::cmp::max(
                vote_policy.quorum.0,
                match &vote_policy.weight_kind {
                    WeightKind::TokenWeight => vote_policy.threshold.to_weight(total_supply),
                    WeightKind::CouncilWeight => vote_policy.threshold.to_weight(
                        council_info
                            .members
                            .get_council_size()
                            .expect("ERR_UNSUPPORTED_COUNCIL") as Balance,
                    ),
                },
            );
            // Check if there is anything voted above the threshold specified by policy for given council.
            let vote_counts = proposal
                .vote_counts
                .get(&council_name)
                .expect("ERR_MISSING_COUNCIL");
            if vote_counts[Vote::Approve as usize] >= threshold {
                return ProposalStatus::Approved;
            } else if vote_counts[Vote::Reject as usize] >= threshold {
                return ProposalStatus::Rejected;
            } else if vote_counts[Vote::Remove as usize] >= threshold {
                return ProposalStatus::Removed;
            } else {
                // continue to next council.
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
