use std::convert::TryInto;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{Base64VecU8, WrappedTimestamp};
use near_sdk::{AccountId, Balance, Gas, PromiseOrValue};

use crate::policy::UserInfo;
use crate::types::{
    ext_fungible_token, upgrade_self, Action, Config, BASE_TOKEN, GAS_FOR_FT_TRANSFER,
    GAS_FOR_UPGRADE_REMOTE_PROMISE, NO_DEPOSIT, ONE_YOCTO_NEAR,
};
use crate::*;
use std::collections::HashMap;

/// Status of a proposal.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
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
}

/// Kinds of proposals, doing different action.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(feature = "test", derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum ProposalKind {
    /// Change the DAO config.
    ChangeConfig { config: Config },
    /// Change the full policy.
    ChangePolicy { policy: VersionedPolicy },
    /// Add member to given role in the policy. This is short cut to updating the whole policy.
    AddMemberToRole { member_id: AccountId, role: String },
    /// Remove member to given role in the policy. This is short cut to updating the whole policy.
    RemoveMemberFromRole { member_id: AccountId, role: String },
    FunctionCall {
        receiver_id: AccountId,
        method_name: String,
        args: Base64VecU8,
        deposit: U128,
        gas: Gas,
    },
    /// Upgrade this contract with given hash from blob store.
    UpgradeSelf { hash: Base64VecU8 },
    /// Upgrade another contract, by calling method with the code from given hash from blob store.
    UpgradeRemote {
        receiver_id: AccountId,
        method_name: String,
        hash: Base64VecU8,
    },
    /// Transfers given amount of `token_id` from this DAO to `receiver_id`.
    Transfer {
        token_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    },
    /// Mints new tokens inside this DAO.
    Mint { amount: U128 },
    /// Burns tokens inside this DAO.
    Burn { amount: U128 },
    /// Add new bounty.
    AddBounty { bounty: Bounty },
    /// Indicates that given bounty is done by given user.
    BountyDone {
        bounty_id: u64,
        receiver_id: AccountId,
    },
    /// Just a signaling vote, with no execution.
    Vote,
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
            ProposalKind::Mint { .. } => "mint",
            ProposalKind::Burn { .. } => "burn",
            ProposalKind::AddBounty { .. } => "add_bounty",
            ProposalKind::BountyDone { .. } => "bounty_done",
            ProposalKind::Vote => "vote",
        }
    }
}

/// Votes recorded in the proposal.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
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
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(feature = "test", derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct Proposal {
    /// Original proposer.
    pub proposer: AccountId,
    /// Description of this proposal.
    pub description: String,
    /// Kind of proposal with relevant information.
    pub kind: ProposalKind,
    /// Current status of the proposal.
    pub status: ProposalStatus,
    /// Count of votes per decision: yes / no / spam.
    pub vote_counts: [Balance; 3],
    /// Map of who voted and how.
    pub votes: HashMap<AccountId, Vote>,
    /// Submission time (for voting period).
    pub submission_time: WrappedTimestamp,
}

impl Proposal {
    /// Adds vote of the given user with given `amount` of weight. If user already voted, fails.
    pub fn update_votes(&mut self, account_id: AccountId, vote: Vote, amount: Balance) {
        self.vote_counts[vote.clone() as usize] += amount;
        assert!(
            self.votes.insert(account_id, vote).is_none(),
            "ERR_ALREADY_VOTED"
        );
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ProposalInput {
    /// Description of this proposal.
    pub description: String,
    /// Kind of proposal with relevant information.
    pub kind: ProposalKind,
}

impl From<ProposalInput> for Proposal {
    fn from(input: ProposalInput) -> Self {
        Self {
            proposer: env::predecessor_account_id(),
            description: input.description,
            kind: input.kind,
            status: ProposalStatus::InProgress,
            vote_counts: [0; 3],
            votes: HashMap::default(),
            submission_time: WrappedTimestamp::from(env::block_timestamp()),
        }
    }
}

impl Contract {
    /// Execute payout of given token to given user.
    pub(crate) fn internal_payout(
        &mut self,
        token_id: &AccountId,
        receiver_id: &AccountId,
        amount: Balance,
    ) -> PromiseOrValue<()> {
        if token_id == &env::current_account_id() {
            self.token
                .internal_withdraw(&env::current_account_id(), amount);
            self.token.internal_deposit(&receiver_id, amount);
            PromiseOrValue::Value(())
        } else if token_id == BASE_TOKEN {
            Promise::new(receiver_id.clone()).transfer(amount).into()
        } else {
            ext_fungible_token::ft_transfer(
                receiver_id.clone(),
                U128(amount),
                None,
                &token_id,
                ONE_YOCTO_NEAR,
                GAS_FOR_FT_TRANSFER,
            )
            .into()
        }
    }

    /// Executes given proposal and updates the contract's state.
    fn internal_execute_proposal(
        &mut self,
        policy: &Policy,
        proposal: &Proposal,
    ) -> PromiseOrValue<()> {
        // Return the proposal bond.
        Promise::new(proposal.proposer.clone()).transfer(policy.proposal_bond.0);
        match &proposal.kind {
            ProposalKind::ChangeConfig { config } => {
                self.data_mut().config.set(config);
                PromiseOrValue::Value(())
            }
            ProposalKind::ChangePolicy { policy } => {
                self.data_mut().policy.set(policy);
                PromiseOrValue::Value(())
            }
            ProposalKind::AddMemberToRole { member_id, role } => {
                let mut new_policy = policy.clone();
                new_policy.add_member_to_role(role, member_id);
                self.data_mut()
                    .policy
                    .set(&VersionedPolicy::Current(new_policy));
                PromiseOrValue::Value(())
            }
            ProposalKind::RemoveMemberFromRole { member_id, role } => {
                let mut new_policy = policy.clone();
                new_policy.remove_member_from_role(role, member_id);
                self.data_mut()
                    .policy
                    .set(&VersionedPolicy::Current(new_policy));
                PromiseOrValue::Value(())
            }
            ProposalKind::FunctionCall {
                receiver_id,
                method_name,
                args,
                deposit,
                gas,
            } => Promise::new(receiver_id.clone())
                .function_call(
                    method_name.clone().into_bytes(),
                    args.clone().into(),
                    deposit.0,
                    *gas,
                )
                .into(),
            ProposalKind::UpgradeSelf { hash } => {
                upgrade_self(&hash.0);
                PromiseOrValue::Value(())
            }
            ProposalKind::UpgradeRemote {
                receiver_id,
                method_name,
                hash,
            } => {
                let code = env::storage_read(&hash.0).expect("ERR_NO_CODE_STAGED");
                Promise::new(receiver_id.clone())
                    .function_call(
                        method_name.clone().into_bytes(),
                        code,
                        NO_DEPOSIT,
                        env::prepaid_gas() - env::used_gas() - GAS_FOR_UPGRADE_REMOTE_PROMISE,
                    )
                    .into()
            }
            ProposalKind::Transfer {
                token_id,
                receiver_id,
                amount,
            } => self.internal_payout(token_id, receiver_id, amount.0),
            ProposalKind::Mint { amount } => {
                self.token
                    .internal_deposit(&env::current_account_id(), amount.0);
                PromiseOrValue::Value(())
            }
            ProposalKind::Burn { amount } => {
                self.token
                    .internal_withdraw(&env::current_account_id(), amount.0);
                PromiseOrValue::Value(())
            }
            ProposalKind::AddBounty { bounty } => {
                self.internal_add_bounty(bounty.clone());
                PromiseOrValue::Value(())
            }
            ProposalKind::BountyDone {
                bounty_id,
                receiver_id,
            } => self.internal_execute_bounty_payout(*bounty_id, receiver_id, true),
            ProposalKind::Vote => PromiseOrValue::Value(()),
        }
    }

    /// Process rejecting proposal.
    fn internal_reject_proposal(
        &mut self,
        policy: &Policy,
        proposal: &Proposal,
        return_bond: bool,
    ) -> PromiseOrValue<()> {
        if return_bond {
            // Return bond to the proposer.
            Promise::new(proposal.proposer.clone()).transfer(policy.proposal_bond.0);
        }
        match &proposal.kind {
            ProposalKind::BountyDone {
                bounty_id,
                receiver_id,
            } => self.internal_execute_bounty_payout(*bounty_id, receiver_id, false),
            _ => PromiseOrValue::Value(()),
        }
    }

    pub(crate) fn internal_user_info(&self) -> UserInfo {
        let account_id = env::predecessor_account_id();
        UserInfo {
            amount: self.token.accounts.get(&account_id),
            account_id,
        }
    }
}

#[near_bindgen]
impl Contract {
    /// Add proposal to this DAO.
    #[payable]
    pub fn add_proposal(&mut self, proposal: ProposalInput) -> u64 {
        // 0. validate bond attached.
        // TODO: consider bond in the token of this DAO.
        let policy = self.data().policy.get().unwrap().to_policy();
        assert!(
            env::attached_deposit() >= policy.proposal_bond.0,
            "ERR_MIN_BOND"
        );

        // 1. validate proposal.
        // TODO: ???

        // 2. check permission of caller to add proposal.
        assert!(
            policy.can_execute_action(
                self.internal_user_info(),
                &proposal.kind,
                &Action::AddProposal
            ),
            "ERR_PERMISSION_DENIED"
        );

        // 3. actually add proposal to current list.
        let id = self.data().last_proposal_id;
        self.data_mut().proposals.insert(&id, &proposal.into());
        self.data_mut().last_proposal_id += 1;
        id
    }

    /// Act on given proposal by id, if permissions allow.
    pub fn act_proposal(&mut self, id: u64, action: Action) {
        let mut proposal = self.data_mut().proposals.get(&id).expect("ERR_NO_PROPOSAL");
        let policy = self.data().policy.get().unwrap().to_policy();
        // Check permissions for given action.
        assert!(
            policy.can_execute_action(
                self.internal_user_info(),
                &proposal.kind,
                &Action::RemoveProposal
            ),
            "ERR_PERMISSION_DENIED"
        );
        let sender_id = env::predecessor_account_id();
        // Update proposal given action. Returns true if should be updated in storage.
        let update = match action {
            Action::AddProposal => env::panic(b"ERR_WRONG_ACTION"),
            Action::RemoveProposal => {
                self.data_mut().proposals.remove(&id);
                false
            }
            Action::VoteApprove | Action::VoteReject | Action::VoteRemove => {
                assert_eq!(
                    proposal.status,
                    ProposalStatus::InProgress,
                    "ERR_PROPOSAL_NOT_IN_PROGRESS"
                );
                let amount = if policy.is_token_weighted(&proposal.kind) {
                    self.ft_balance_of(sender_id.clone().try_into().unwrap()).0
                } else {
                    1
                };
                proposal.update_votes(sender_id, Vote::from(action), amount);
                // Updates proposal status with new votes using the policy.
                proposal.status = policy.proposal_status(&proposal, self.ft_total_supply().0);
                if proposal.status == ProposalStatus::Approved {
                    self.internal_execute_proposal(&policy, &proposal);
                    true
                } else if proposal.status == ProposalStatus::Removed {
                    self.internal_reject_proposal(&policy, &proposal, false);
                    self.data_mut().proposals.remove(&id);
                    false
                } else if proposal.status == ProposalStatus::Rejected {
                    self.internal_reject_proposal(&policy, &proposal, true);
                    true
                } else {
                    // Still in progress or expired.
                    true
                }
            }
            Action::Finalize => {
                proposal.status = policy.proposal_status(&proposal, self.ft_total_supply().0);
                assert_eq!(
                    proposal.status,
                    ProposalStatus::Expired,
                    "ERR_PROPOSAL_NOT_EXPIRED"
                );
                self.internal_reject_proposal(&policy, &proposal, true);
                true
            }
            Action::MoveToHub => false,
        };
        if update {
            self.data_mut().proposals.insert(&id, &proposal);
        }
    }
}
