use std::collections::HashSet;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{self, Deserialize, Serialize};
use near_sdk::{env, AccountId, Balance};
use regex::Regex;

use crate::proposals::{Proposal, ProposalKind, ProposalStatus};
use crate::types::Action;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum RoleKind {
    /// Matches everyone, who is not matched by other roles.
    Everyone,
    /// Member: has non zero balance on this DAOs' token.
    Member,
    /// Member with at least given balance (must be non 0).
    MemberBalance(Balance),
    /// Set of accounts.
    Group(Vec<AccountId>),
    /// Set of accounts matches by regex.
    Regex(String),
}

impl RoleKind {
    /// Checks if user matches given role.
    pub fn match_user(&self, user: &UserInfo) -> bool {
        match self {
            RoleKind::Everyone => true,
            RoleKind::Member => user.amount.is_some(),
            RoleKind::MemberBalance(amount) => user.amount.unwrap_or_default() >= *amount,
            RoleKind::Group(accounts) => accounts.contains(&user.account_id),
            RoleKind::Regex(regex) => Regex::new(regex)
                .expect("Invalid regex")
                .is_match(&user.account_id),
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct RolePermission {
    name: String,
    kind: RoleKind,
    /// Set of actions on which proposals that this role is allowed to execute.
    /// <proposal_kind>:<action>
    permissions: HashSet<String>,
}

pub struct UserInfo {
    pub account_id: AccountId,
    pub amount: Option<Balance>,
}

/// Defines voting / decision making policy of this DAO.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Policy {
    roles: Vec<RolePermission>,
}

impl Default for Policy {
    /// Defines default policy:
    ///     - everyone can add proposals
    ///     - group consisting of the call can do all actions
    fn default() -> Self {
        Self {
            roles: vec![
                RolePermission {
                    name: "all".to_string(),
                    kind: RoleKind::Everyone,
                    permissions: vec!["*:add_proposal".to_string()].into_iter().collect(),
                },
                RolePermission {
                    name: "council".to_string(),
                    kind: RoleKind::Group(vec![env::predecessor_account_id()]),
                    permissions: vec!["*:*".to_string()].into_iter().collect(),
                },
            ],
        }
    }
}

impl Policy {
    fn get_user_permissions(&self, user: UserInfo) -> HashSet<String> {
        let mut result = HashSet::default();
        for role in self.roles.iter() {
            if role.kind.match_user(&user) {
                result = result.union(&role.permissions).cloned().collect();
            }
        }
        result
    }

    /// Can given user execute given action on this proposal.
    pub fn can_execute_action(
        &self,
        user: UserInfo,
        proposal_kind: &ProposalKind,
        action: &Action,
    ) -> bool {
        let permissions = self.get_user_permissions(user);
        permissions.contains(&format!(
            "{}:{}",
            proposal_kind.to_policy_label(),
            action.to_policy_label()
        )) || permissions.contains(&format!("{}:*", proposal_kind.to_policy_label()))
            || permissions.contains(&format!("*:{}", action.to_policy_label()))
            || permissions.contains("*:*")
    }

    /// Get proposal status for given proposal.
    /// Usually is called after changing it's state.
    pub fn proposal_status(&self, proposal: &Proposal) -> ProposalStatus {
        // TODO: implement status transition.
        proposal.status.clone()
    }
}
