use std::collections::{HashMap, VecDeque};

use ext_fungible_token::ext_fungible_token;
use near_sdk::json_types::{Base64VecU8, U128, U64};
use near_sdk::{log, AccountId, Gas, PromiseOrValue};

use crate::action_log::ActionLog;
use crate::policy::UserInfo;
use crate::types::{
    convert_old_to_new_token, Action, Config, OldAccountId, GAS_FOR_FT_TRANSFER, OLD_BASE_TOKEN,
    ONE_YOCTO_NEAR,
};
use crate::upgrade::{upgrade_remote, upgrade_using_factory};
use crate::*;

/// Status of a proposal.
#[derive(Clone, PartialEq, Debug)]
#[near(serializers=[borsh, json])]
pub enum ProposalStatus {
    InProgress,
    /// If quorum voted yes, this proposal is successfully approved.
    Approved,
    /// If quorum voted no, this proposal is rejected. Bond is returned.
    Rejected,
    /// If quorum voted to remove (e.g. spam), this proposal is rejected and bond is not returned.
    /// Interfaces shouldn't show removed proposals.
    Removed,
    /// Expired after period of time.
    Expired,
    /// If proposal was moved to Hub or somewhere else.
    Moved,
    /// If proposal has failed when finalizing. Allowed to re-finalize again to either expire or approved.
    Failed,
}

/// Function call arguments.

#[derive(PartialEq, Clone)]
#[near(serializers=[borsh, json])]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
pub struct ActionCall {
    method_name: String,
    args: Base64VecU8,
    deposit: U128,
    gas: U64,
}

/// Function call arguments.

#[derive(PartialEq, Clone)]
#[near(serializers=[borsh, json])]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
pub struct PolicyParameters {
    pub proposal_bond: Option<U128>,
    pub proposal_period: Option<U64>,
    pub bounty_bond: Option<U128>,
    pub bounty_forgiveness_period: Option<U64>,
}

/// Kinds of proposals, doing different action.
#[derive(PartialEq, Clone)]
#[near(serializers=[borsh, json])]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
pub enum ProposalKind {
    /// Change the DAO config.
    ChangeConfig { config: Config },
    /// Change the full policy.
    ChangePolicy { policy: VersionedPolicy },
    /// Add member to given role in the policy. This is short cut to updating the whole policy.
    AddMemberToRole { member_id: AccountId, role: String },
    /// Remove member to given role in the policy. This is short cut to updating the whole policy.
    RemoveMemberFromRole { member_id: AccountId, role: String },
    /// Calls `receiver_id` with list of method names in a single promise.
    /// Allows this contract to execute any arbitrary set of actions in other contracts.
    FunctionCall {
        receiver_id: AccountId,
        actions: Vec<ActionCall>,
    },
    /// Upgrade this contract with given hash from blob store.
    UpgradeSelf { hash: Base58CryptoHash },
    /// Upgrade another contract, by calling method with the code from given hash from blob store.
    UpgradeRemote {
        receiver_id: AccountId,
        method_name: String,
        hash: Base58CryptoHash,
    },
    /// Transfers given amount of `token_id` from this DAO to `receiver_id`.
    /// If `msg` is not None, calls `ft_transfer_call` with given `msg`. Fails if this base token.
    /// For `ft_transfer` and `ft_transfer_call` `memo` is the `description` of the proposal.
    Transfer {
        /// Can be "" for $NEAR or a valid account id.
        token_id: OldAccountId,
        receiver_id: AccountId,
        amount: U128,
        msg: Option<String>,
    },
    /// Sets staking contract. Can only be proposed if staking contract is not set yet.
    SetStakingContract { staking_id: AccountId },
    /// Add new bounty.
    AddBounty { bounty: Bounty },
    /// Indicates that given bounty is done by given user.
    BountyDone {
        bounty_id: u64,
        receiver_id: AccountId,
    },
    /// Just a signaling vote, with no execution.
    Vote,
    /// Change information about factory and auto update.
    FactoryInfoUpdate { factory_info: FactoryInfo },
    /// Add new role to the policy. If the role already exists, update it. This is short cut to updating the whole policy.
    ChangePolicyAddOrUpdateRole { role: RolePermission },
    /// Remove role from the policy. This is short cut to updating the whole policy.
    ChangePolicyRemoveRole { role: String },
    /// Update the default vote policy from the policy. This is short cut to updating the whole policy.
    ChangePolicyUpdateDefaultVotePolicy { vote_policy: VotePolicy },
    /// Update the parameters from the policy. This is short cut to updating the whole policy.
    ChangePolicyUpdateParameters { parameters: PolicyParameters },
}

impl ProposalKind {
    /// Returns label of policy for given type of proposal.
    pub fn to_policy_label(&self) -> &str {
        match self {
            ProposalKind::ChangeConfig { .. } => "config",
            ProposalKind::ChangePolicy { .. } => "policy",
            ProposalKind::AddMemberToRole { .. } => "add_member_to_role",
            ProposalKind::RemoveMemberFromRole { .. } => "remove_member_from_role",
            ProposalKind::FunctionCall { .. } => "call",
            ProposalKind::UpgradeSelf { .. } => "upgrade_self",
            ProposalKind::UpgradeRemote { .. } => "upgrade_remote",
            ProposalKind::Transfer { .. } => "transfer",
            ProposalKind::SetStakingContract { .. } => "set_vote_token",
            ProposalKind::AddBounty { .. } => "add_bounty",
            ProposalKind::BountyDone { .. } => "bounty_done",
            ProposalKind::Vote => "vote",
            ProposalKind::FactoryInfoUpdate { .. } => "factory_info_update",
            ProposalKind::ChangePolicyAddOrUpdateRole { .. } => "policy_add_or_update_role",
            ProposalKind::ChangePolicyRemoveRole { .. } => "policy_remove_role",
            ProposalKind::ChangePolicyUpdateDefaultVotePolicy { .. } => {
                "policy_update_default_vote_policy"
            }
            ProposalKind::ChangePolicyUpdateParameters { .. } => "policy_update_parameters",
        }
    }
}

/// Votes recorded in the proposal.
#[derive(Clone, Debug)]
#[near(serializers=[borsh(use_discriminant=true),json])]
pub enum Vote {
    Approve = 0x0,
    Reject = 0x1,
    Remove = 0x2,
}

impl From<Action> for Vote {
    fn from(action: Action) -> Self {
        match action {
            Action::VoteApprove => Vote::Approve,
            Action::VoteReject => Vote::Reject,
            Action::VoteRemove => Vote::Remove,
            _ => unreachable!(),
        }
    }
}

/// Proposal that are sent to this DAO.
#[near(serializers=[borsh, json])]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(Clone)]
pub struct ProposalV0 {
    /// Original proposer.
    pub proposer: AccountId,
    /// Description of this proposal.
    pub description: String,
    /// Kind of proposal with relevant information.
    pub kind: ProposalKind,
    /// Current status of the proposal.
    pub status: ProposalStatus,
    /// Count of votes per role per decision: yes / no / spam.
    pub vote_counts: HashMap<String, [U128; 3]>,
    /// Map of who voted and how.
    pub votes: HashMap<AccountId, Vote>,
    /// Submission time (for voting period).
    pub submission_time: U64,
}

#[near(serializers=[borsh, json])]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[derive(Clone)]
pub struct ProposalV1 {
    /// Original proposer.
    pub proposer: AccountId,
    /// Description of this proposal.
    pub description: String,
    /// Kind of proposal with relevant information.
    pub kind: ProposalKind,
    /// Current status of the proposal.
    pub status: ProposalStatus,
    /// Count of votes per role per decision: yes / no / spam.
    pub vote_counts: HashMap<String, [U128; 3]>,
    /// Map of who voted and how.
    pub votes: HashMap<AccountId, Vote>,
    /// Submission time (for voting period).
    pub submission_time: U64,
    /// Last actions log
    pub last_actions_log: Option<VecDeque<ActionLog>>,
}

impl From<ProposalV0> for ProposalV1 {
    fn from(v0: ProposalV0) -> Self {
        ProposalV1 {
            proposer: v0.proposer.clone(),
            description: v0.description.clone(),
            kind: v0.kind.clone(),
            status: v0.status.clone(),
            vote_counts: v0.vote_counts.clone(),
            votes: v0.votes.clone(),
            submission_time: v0.submission_time,
            last_actions_log: Some(VecDeque::new()),
        }
    }
}

#[derive(Clone)]
#[near(serializers=[borsh, json])]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(untagged)]
pub enum VersionedProposal {
    V0(ProposalV0),
    V1(ProposalV1),
}

impl From<VersionedProposal> for ProposalV0 {
    fn from(v: VersionedProposal) -> Self {
        match v {
            VersionedProposal::V0(p) => p,
            _ => unimplemented!(),
        }
    }
}

impl From<VersionedProposal> for ProposalV1 {
    fn from(v: VersionedProposal) -> Self {
        match v {
            VersionedProposal::V0(p) => p.into(),
            VersionedProposal::V1(p) => p,
        }
    }
}

impl VersionedProposal {
    /// Adds vote of the given user with given `amount` of weight. If user already voted, fails.
    pub fn update_votes(
        &mut self,
        account_id: &AccountId,
        roles: &[String],
        vote: Vote,
        policy: &Policy,
        user_weight: Balance,
    ) {
        for role in roles {
            let amount = if policy.is_token_weighted(
                role,
                &self.latest_version_ref().kind.to_policy_label().to_string(),
            ) {
                user_weight
            } else {
                1
            };
            self.update_counts(role.clone(), vote.clone(), amount);
        }
        self.insert_vote(account_id, vote);
    }

    pub fn latest_version(self) -> ProposalV1 {
        self.into()
    }

    pub fn latest_version_ref(&self) -> ProposalV1 {
        match self {
            VersionedProposal::V0(p) => ProposalV1::from(p.clone()),
            VersionedProposal::V1(p) => p.clone(),
        }
    }

    pub fn update_counts(&mut self, role: String, vote: Vote, amount: u128) {
        let defaults = [U128::from(0); 3];

        match self {
            VersionedProposal::V0(p) => {
                let vote_counted =
                    p.vote_counts.entry(role.clone()).or_insert(defaults)[vote.clone() as usize].0
                        + amount;
                p.vote_counts
                    .entry(role)
                    .and_modify(|votes| votes[vote.clone() as usize] = vote_counted.into());
            }
            VersionedProposal::V1(p) => {
                let vote_counted =
                    p.vote_counts.entry(role.clone()).or_insert(defaults)[vote.clone() as usize].0
                        + amount;
                p.vote_counts
                    .entry(role)
                    .and_modify(|votes| votes[vote.clone() as usize] = vote_counted.into());
            }
        }
    }

    pub fn insert_vote(&mut self, account_id: &AccountId, vote: Vote) {
        match self {
            VersionedProposal::V0(p) => {
                assert!(
                    p.votes.insert(account_id.clone(), vote).is_none(),
                    "ERR_ALREADY_VOTED"
                )
            }
            VersionedProposal::V1(p) => {
                assert!(
                    p.votes.insert(account_id.clone(), vote).is_none(),
                    "ERR_ALREADY_VOTED"
                )
            }
        }
    }
}

#[near(serializers=[json])]
pub struct ProposalInput {
    /// Description of this proposal.
    pub description: String,
    /// Kind of proposal with relevant information.
    pub kind: ProposalKind,
}

impl From<ProposalInput> for VersionedProposal {
    fn from(input: ProposalInput) -> Self {
        VersionedProposal::V1(ProposalV1 {
            proposer: env::predecessor_account_id(),
            description: input.description,
            kind: input.kind,
            status: ProposalStatus::InProgress,
            vote_counts: HashMap::default(),
            votes: HashMap::default(),
            submission_time: U64::from(env::block_timestamp()),
            last_actions_log: Some(VecDeque::new()),
        })
    }
}

impl Contract {
    /// Execute payout of given token to given user.
    pub(crate) fn internal_payout(
        &mut self,
        token_id: &Option<AccountId>,
        receiver_id: &AccountId,
        amount: Balance,
        memo: String,
        msg: Option<String>,
    ) -> PromiseOrValue<()> {
        if token_id.is_none() {
            Promise::new(receiver_id.clone())
                .transfer(NearToken::from_yoctonear(amount))
                .into()
        } else {
            if let Some(msg) = msg {
                ext_fungible_token::ext(token_id.as_ref().unwrap().clone())
                    .with_attached_deposit(ONE_YOCTO_NEAR)
                    .with_static_gas(GAS_FOR_FT_TRANSFER)
                    .ft_transfer_call(receiver_id.clone(), U128(amount), Some(memo), msg)
            } else {
                ext_fungible_token::ext(token_id.as_ref().unwrap().clone())
                    .with_attached_deposit(ONE_YOCTO_NEAR)
                    .with_static_gas(GAS_FOR_FT_TRANSFER)
                    .ft_transfer(receiver_id.clone(), U128(amount), Some(memo))
            }
            .into()
        }
    }

    fn internal_return_bonds(&mut self, policy: &Policy, proposal: &VersionedProposal) -> Promise {
        let proposal_data = match proposal {
            VersionedProposal::V0(p) => ProposalV1 {
                proposer: p.proposer.clone(),
                description: p.description.clone(),
                kind: p.kind.clone(),
                status: p.status.clone(),
                vote_counts: p.vote_counts.clone(),
                votes: p.votes.clone(),
                submission_time: p.submission_time,
                last_actions_log: Some(VecDeque::new()),
            },
            VersionedProposal::V1(p) => p.clone(),
        };
        match &proposal_data.kind {
            ProposalKind::BountyDone { .. } => {
                self.locked_amount = self
                    .locked_amount
                    .saturating_sub(NearToken::from_yoctonear(policy.bounty_bond.0));
                Promise::new(proposal_data.proposer.clone())
                    .transfer(NearToken::from_yoctonear(policy.bounty_bond.0));
            }
            _ => {}
        }

        self.locked_amount = self
            .locked_amount
            .saturating_sub(NearToken::from_yoctonear(policy.proposal_bond.0));
        Promise::new(proposal_data.proposer.clone())
            .transfer(NearToken::from_yoctonear(policy.proposal_bond.0))
    }

    /// Executes given proposal and updates the contract's state.
    fn internal_execute_proposal(
        &mut self,
        policy: &Policy,
        proposal: &ProposalV1,
        proposal_id: u64,
    ) -> PromiseOrValue<()> {
        let result = match &proposal.kind {
            ProposalKind::ChangeConfig { config } => {
                self.config.set(config);
                PromiseOrValue::Value(())
            }
            ProposalKind::ChangePolicy { policy } => {
                self.policy.set(policy);
                PromiseOrValue::Value(())
            }
            ProposalKind::AddMemberToRole { member_id, role } => {
                let mut new_policy = policy.clone();
                new_policy.add_member_to_role(role, &member_id.clone().into());
                self.policy.set(&VersionedPolicy::Current(new_policy));
                PromiseOrValue::Value(())
            }
            ProposalKind::RemoveMemberFromRole { member_id, role } => {
                let mut new_policy = policy.clone();
                new_policy.remove_member_from_role(role, &member_id.clone().into());
                self.policy.set(&VersionedPolicy::Current(new_policy));
                PromiseOrValue::Value(())
            }
            ProposalKind::FunctionCall {
                receiver_id,
                actions,
            } => {
                let mut promise = Promise::new(receiver_id.clone().into());
                for action in actions {
                    promise = promise.function_call(
                        action.method_name.clone().into(),
                        action.args.clone().into(),
                        NearToken::from_yoctonear(action.deposit.0),
                        Gas::from_gas(action.gas.0),
                    )
                }
                promise.into()
            }
            ProposalKind::UpgradeSelf { hash } => {
                upgrade_using_factory(hash.clone());
                PromiseOrValue::Value(())
            }
            ProposalKind::UpgradeRemote {
                receiver_id,
                method_name,
                hash,
            } => {
                upgrade_remote(&receiver_id, method_name, &CryptoHash::from(hash.clone()));
                PromiseOrValue::Value(())
            }
            ProposalKind::Transfer {
                token_id,
                receiver_id,
                amount,
                msg,
            } => self.internal_payout(
                &convert_old_to_new_token(token_id),
                &receiver_id,
                amount.0,
                proposal.description.clone(),
                msg.clone(),
            ),
            ProposalKind::SetStakingContract { staking_id } => {
                assert!(self.staking_id.is_none(), "ERR_INVALID_STAKING_CHANGE");
                self.staking_id = Some(staking_id.clone().into());
                PromiseOrValue::Value(())
            }
            ProposalKind::AddBounty { bounty } => {
                self.internal_add_bounty(bounty);
                PromiseOrValue::Value(())
            }
            ProposalKind::BountyDone {
                bounty_id,
                receiver_id,
            } => self.internal_execute_bounty_payout(*bounty_id, &receiver_id.clone().into(), true),
            ProposalKind::Vote => PromiseOrValue::Value(()),
            ProposalKind::FactoryInfoUpdate { factory_info } => {
                internal_set_factory_info(factory_info);
                PromiseOrValue::Value(())
            }
            ProposalKind::ChangePolicyAddOrUpdateRole { role } => {
                let mut new_policy = policy.clone();
                new_policy.add_or_update_role(role);
                self.policy.set(&VersionedPolicy::Current(new_policy));
                PromiseOrValue::Value(())
            }
            ProposalKind::ChangePolicyRemoveRole { role } => {
                let mut new_policy = policy.clone();
                new_policy.remove_role(role);
                self.policy.set(&VersionedPolicy::Current(new_policy));
                PromiseOrValue::Value(())
            }
            ProposalKind::ChangePolicyUpdateDefaultVotePolicy { vote_policy } => {
                let mut new_policy = policy.clone();
                new_policy.update_default_vote_policy(vote_policy);
                self.policy.set(&VersionedPolicy::Current(new_policy));
                PromiseOrValue::Value(())
            }
            ProposalKind::ChangePolicyUpdateParameters { parameters } => {
                let mut new_policy = policy.clone();
                new_policy.update_parameters(parameters);
                self.policy.set(&VersionedPolicy::Current(new_policy));
                PromiseOrValue::Value(())
            }
        };
        match result {
            PromiseOrValue::Promise(promise) => promise
                .then(
                    Self::ext(env::current_account_id())
                        .with_static_gas(GAS_FOR_FT_TRANSFER)
                        .on_proposal_callback(proposal_id),
                )
                .into(),
            PromiseOrValue::Value(()) => {
                let versioned = VersionedProposal::V1(proposal.clone());
                self.internal_return_bonds(&policy, &versioned).into()
            }
        }
    }

    pub(crate) fn internal_callback_proposal_success(
        &mut self,
        proposal: &mut VersionedProposal,
    ) -> PromiseOrValue<()> {
        let policy = self.policy.get().unwrap().to_policy();
        let proposal_data = proposal.latest_version_ref();
        if let ProposalKind::BountyDone { bounty_id, .. } = &proposal_data.kind {
            let mut bounty: Bounty = self.bounties.get(bounty_id).expect("ERR_NO_BOUNTY").into();
            if bounty.times == 0 {
                self.bounties.remove(bounty_id);
            } else {
                bounty.times -= 1;
                self.bounties
                    .insert(bounty_id, &VersionedBounty::Default(bounty));
            }
        }

        match proposal {
            VersionedProposal::V0(p) => p.status = ProposalStatus::Approved,
            VersionedProposal::V1(p) => p.status = ProposalStatus::Approved,
        }

        self.internal_return_bonds(&policy, &proposal).into()
    }

    pub(crate) fn internal_callback_proposal_fail(
        &mut self,
        proposal: &mut VersionedProposal,
    ) -> PromiseOrValue<()> {
        match proposal {
            VersionedProposal::V0(p) => p.status = ProposalStatus::Failed,
            VersionedProposal::V1(p) => p.status = ProposalStatus::Failed,
        }
        PromiseOrValue::Value(())
    }

    /// Process rejecting proposal.
    fn internal_reject_proposal(
        &mut self,
        policy: &Policy,
        proposal: &VersionedProposal,
        return_bonds: bool,
    ) -> PromiseOrValue<()> {
        if return_bonds {
            // Return bond to the proposer.
            self.internal_return_bonds(policy, proposal);
        }

        let proposal_data = proposal.latest_version_ref();
        match &proposal_data.kind {
            ProposalKind::BountyDone {
                bounty_id,
                receiver_id,
            } => {
                self.internal_execute_bounty_payout(*bounty_id, &receiver_id.clone().into(), false)
            }
            _ => PromiseOrValue::Value(()),
        }
    }

    pub(crate) fn internal_user_info(&self) -> UserInfo {
        let account_id = env::predecessor_account_id();
        UserInfo {
            amount: self.get_user_weight(&account_id),
            account_id,
        }
    }
}

#[near]
impl Contract {
    /// Add proposal to this DAO.
    #[payable]
    pub fn add_proposal(&mut self, proposal: ProposalInput) -> u64 {
        // 0. validate bond attached.
        // TODO: consider bond in the token of this DAO.
        let policy = self.policy.get().unwrap().to_policy();

        assert_eq!(
            env::attached_deposit(),
            NearToken::from_yoctonear(policy.proposal_bond.0),
            "ERR_MIN_BOND"
        );

        // 1. Validate proposal.
        match &proposal.kind {
            ProposalKind::ChangePolicy { policy } => match policy {
                VersionedPolicy::Current(_) => {}
                _ => panic!("ERR_INVALID_POLICY"),
            },
            ProposalKind::Transfer { token_id, msg, .. } => {
                assert!(
                    !(token_id == OLD_BASE_TOKEN) || msg.is_none(),
                    "ERR_BASE_TOKEN_NO_MSG"
                );
            }
            ProposalKind::SetStakingContract { .. } => assert!(
                self.staking_id.is_none(),
                "ERR_STAKING_CONTRACT_CANT_CHANGE"
            ),
            // TODO: add more verifications.
            _ => {}
        };

        // 2. Check permission of caller to add this type of proposal.
        assert!(
            policy
                .can_execute_action(
                    self.internal_user_info(),
                    &proposal.kind,
                    &Action::AddProposal
                )
                .1,
            "ERR_PERMISSION_DENIED"
        );

        // 3. Actually add proposal to the current list of proposals.
        let id = self.last_proposal_id;
        self.proposals
            .insert(&id, &VersionedProposal::from(proposal));
        self.last_proposal_id += 1;
        self.locked_amount = self.locked_amount.saturating_add(env::attached_deposit());
        id
    }

    /// Act on given proposal by id, if permissions allow.
    /// Memo is logged but not stored in the state. Can be used to leave notes or explain the action.
    pub fn act_proposal(
        &mut self,
        id: u64,
        action: Action,
        proposal: ProposalKind,
        memo: Option<String>,
    ) {
        let input_proposal_kind = proposal;
        let mut proposal: VersionedProposal = self.proposals.get(&id).expect("ERR_NO_PROPOSAL");
        let policy = self.policy.get().unwrap().to_policy();

        // Get the proposal data for permission check
        let proposal_data = proposal.latest_version_ref();

        // Check permissions for the given action.
        let (roles, allowed) =
            policy.can_execute_action(self.internal_user_info(), &proposal_data.kind, &action);
        assert!(allowed, "ERR_PERMISSION_DENIED");
        let sender_id = env::predecessor_account_id();

        // Verify propolsal kind
        assert!(
            proposal.latest_version_ref().kind == input_proposal_kind,
            "ERR_WRONG_KIND"
        );
        // Update proposal given action. Returns true if should be updated in storage.
        let update = match action {
            Action::AddProposal => env::panic_str("ERR_WRONG_ACTION"),
            Action::RemoveProposal => {
                self.proposals.remove(&id);
                false
            }
            Action::VoteApprove | Action::VoteReject | Action::VoteRemove => {
                assert!(
                    matches!(
                        proposal.latest_version_ref().status,
                        ProposalStatus::InProgress
                    ),
                    "ERR_PROPOSAL_NOT_READY_FOR_VOTE"
                );
                proposal.update_votes(
                    &sender_id,
                    &roles,
                    Vote::from(action.clone()),
                    &policy,
                    self.get_user_weight(&sender_id),
                );

                // Get new status without cloning the entire proposal
                let new_status =
                    policy.proposal_status(&proposal, roles.clone(), self.total_delegation_amount);

                // Updates proposal status with new votes using the policy.
                match &mut proposal {
                    VersionedProposal::V0(p) => p.status = new_status,
                    VersionedProposal::V1(p) => p.status = new_status,
                };

                // Get updated status by reference
                let status = match &proposal {
                    VersionedProposal::V0(p) => &p.status,
                    VersionedProposal::V1(p) => &p.status,
                };

                if *status == ProposalStatus::Approved {
                    let proposal_v1: ProposalV1 = proposal.clone().into();
                    self.internal_execute_proposal(&policy, &proposal_v1, id);
                    true
                } else if proposal.latest_version_ref().status == ProposalStatus::Removed {
                    self.internal_reject_proposal(&policy, &proposal, false);
                    self.proposals.remove(&id);
                    false
                } else if proposal.latest_version_ref().status == ProposalStatus::Rejected {
                    self.internal_reject_proposal(&policy, &proposal, true);
                    true
                } else {
                    // Still in progress or expired.
                    true
                }
            }
            // There are two cases when proposal must be finalized manually: expired or failed.
            // In case of failed, we just recompute the status and if it still approved, we re-execute the proposal.
            // In case of expired, we reject the proposal and return the bond.
            // Corner cases:
            //  - if proposal expired during the failed state - it will be marked as expired.
            //  - if the number of votes in the group has changed (new members has been added) -
            //      the proposal can loose it's approved state. In this case new proposal needs to be made, this one can only expire.
            Action::Finalize => {
                let new_status = policy.proposal_status(
                    &proposal,
                    policy.roles.iter().map(|r| r.name.clone()).collect(),
                    self.total_delegation_amount,
                );

                match &mut proposal {
                    VersionedProposal::V0(p) => p.status = new_status,
                    VersionedProposal::V1(p) => p.status = new_status,
                };

                let status = match &proposal {
                    VersionedProposal::V0(p) => &p.status,
                    VersionedProposal::V1(p) => &p.status,
                };

                match status {
                    ProposalStatus::Approved => {
                        let proposal_v1: ProposalV1 = proposal.clone().into();
                        self.internal_execute_proposal(&policy, &proposal_v1, id);
                    }
                    ProposalStatus::Expired => {
                        self.internal_reject_proposal(&policy, &proposal, true);
                    }
                    _ => {
                        env::panic_str("ERR_PROPOSAL_NOT_EXPIRED_OR_FAILED");
                    }
                }
                true
            }
            Action::MoveToHub => false,
        };
        if update {
            let mut proposal_v1: ProposalV1 = proposal.clone().into();
            self.internal_log_action(id, action, &mut proposal_v1);
            self.proposals
                .insert(&id, &VersionedProposal::V1(proposal_v1));
        }
        if let Some(memo) = memo {
            log!("Memo: {}", memo);
        }
    }

    /// Receiving callback after the proposal has been finalized.
    /// If successful, returns bond money to the proposal originator.
    /// If the proposal execution failed (funds didn't transfer or function call failure),
    /// move proposal to "Failed" state.
    #[private]
    pub fn on_proposal_callback(&mut self, proposal_id: u64) -> PromiseOrValue<()> {
        let mut proposal: VersionedProposal =
            self.proposals.get(&proposal_id).expect("ERR_NO_PROPOSAL");
        assert_eq!(
            env::promise_results_count(),
            1,
            "ERR_UNEXPECTED_CALLBACK_PROMISES"
        );
        let result = match env::promise_result(0) {
            PromiseResult::Successful(_) => self.internal_callback_proposal_success(&mut proposal),
            PromiseResult::Failed => self.internal_callback_proposal_fail(&mut proposal),
        };
        self.proposals
            .insert(&proposal_id, &VersionedProposal::V1(proposal.into()));
        result
    }
}
