use std::convert::TryInto;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::Base64VecU8;
use near_sdk::{AccountId, Balance, Gas, PromiseOrValue};

use crate::policy::UserInfo;
use crate::types::{
    ext_fungible_token, Action, Config, BASE_TOKEN, GAS_FOR_FT_TRANSFER, ONE_YOCTO_NEAR,
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
    /// If proposal was moved to Hub or somewhere else.
    Moved,
}

/// Kinds of proposals, doing different action.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum ProposalKind {
    ChangeConfig {
        config: Config,
    },
    ChangePolicy {
        policy: Policy,
    },
    FunctionCall {
        receiver_id: AccountId,
        method_name: String,
        args: Base64VecU8,
        deposit: Balance,
        gas: Gas,
    },
    Upgrade,
    /// Transfers given amount of `token_id` from this DAO to `receiver_id`.
    Transfer {
        token_id: AccountId,
        receiver_id: AccountId,
        amount: Balance,
    },
    /// Mints new tokens inside this DAO.
    Mint {
        amount: Balance,
    },
    /// Burns tokens inside this DAO.
    Burn {
        amount: Balance,
    },
}

impl ProposalKind {
    /// Returns label of policy for given type of proposal.
    pub fn to_policy_label(&self) -> &str {
        match self {
            ProposalKind::ChangeConfig { .. } => "config",
            ProposalKind::ChangePolicy { .. } => "policy",
            ProposalKind::FunctionCall { .. } => "call",
            ProposalKind::Upgrade { .. } => "upgrade",
            ProposalKind::Transfer { .. } => "transfer",
            ProposalKind::Mint { .. } => "mint",
            ProposalKind::Burn { .. } => "burn",
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
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug)]
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
}

impl Proposal {
    /// Update count and map of votes.
    pub fn update_votes(&mut self, account_id: AccountId, vote: Vote, amount: Balance) {
        self.vote_counts[vote.clone() as usize] += amount;
        self.votes.insert(account_id, vote);
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
        }
    }
}

#[near_bindgen]
impl Contract {
    /// Executes given proposal and updates the contract's state.
    fn internal_execute_proposal(&mut self, proposal: &Proposal) -> PromiseOrValue<()> {
        match &proposal.kind {
            ProposalKind::ChangeConfig { config } => {
                self.config = config.clone();
                PromiseOrValue::Value(())
            }
            ProposalKind::ChangePolicy { policy } => {
                self.policy = policy.clone();
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
                    *deposit,
                    *gas,
                )
                .into(),
            ProposalKind::Upgrade => {
                env::storage_remove(KEY_STAGE_CODE);
                let code = env::storage_get_evicted().expect("ERR_NO_CODE_STAGED");
                Promise::new(env::current_account_id())
                    .deploy_contract(code)
                    .into()
            }
            ProposalKind::Transfer {
                token_id,
                receiver_id,
                amount,
            } => {
                if token_id == &env::current_account_id() {
                    self.token
                        .internal_withdraw(&env::current_account_id(), *amount);
                    self.token.internal_deposit(&receiver_id, *amount);
                    PromiseOrValue::Value(())
                } else if token_id == BASE_TOKEN {
                    Promise::new(receiver_id.clone()).transfer(*amount).into()
                } else {
                    ext_fungible_token::ft_transfer(
                        receiver_id.clone(),
                        U128(*amount),
                        None,
                        &token_id,
                        ONE_YOCTO_NEAR,
                        GAS_FOR_FT_TRANSFER,
                    )
                    .into()
                }
            }
            ProposalKind::Mint { amount } => {
                self.token
                    .internal_deposit(&env::current_account_id(), *amount);
                PromiseOrValue::Value(())
            }
            ProposalKind::Burn { amount } => {
                self.token
                    .internal_withdraw(&env::current_account_id(), *amount);
                PromiseOrValue::Value(())
            }
        }
    }

    /// Process rejecting proposal.
    fn internal_reject_proposal(&mut self, proposal: &Proposal) -> PromiseOrValue<()> {
        match &proposal.kind {
            ProposalKind::Upgrade => {
                env::storage_remove(KEY_STAGE_CODE);
                PromiseOrValue::Value(())
            }
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

    /// Add proposal to this DAO.
    #[payable]
    pub fn add_proposal(&mut self, proposal: ProposalInput) -> u64 {
        // 0. validate bond attached.
        // TODO: consider bond in the token of this DAO.
        assert!(env::attached_deposit() >= self.config.bond.0);

        // 1. validate proposal.
        // TODO: ???

        // 2. check permission of caller to add proposal.
        assert!(
            self.policy.can_execute_action(
                self.internal_user_info(),
                &proposal.kind,
                &Action::AddProposal
            ),
            "ERR_PERMISSION_DENIED"
        );

        // 3. actually add proposal to current list.
        self.proposals
            .insert(&self.last_proposal_id, &proposal.into());
        self.last_proposal_id += 1;
        self.last_proposal_id - 1
    }

    /// Act on given proposal by id, if permissions allow.
    pub fn act_proposal(&mut self, id: u64, action: Action) {
        let mut proposal = self.proposals.get(&id).expect("ERR_NO_PROPOSAL");
        // Check permissions for given action.
        assert!(
            self.policy.can_execute_action(
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
                self.proposals.remove(&id);
                false
            }
            Action::VoteApprove | Action::VoteReject | Action::VoteRemove => {
                assert_eq!(
                    proposal.status,
                    ProposalStatus::InProgress,
                    "ERR_PROPOSAL_NOT_IN_PROGRESS"
                );
                let amount = if self.policy.is_token_weighted(&proposal.kind) {
                    self.ft_balance_of(sender_id.clone().try_into().unwrap()).0
                } else {
                    1
                };
                proposal.update_votes(sender_id, Vote::from(action), amount);
                // Updates proposal status with new votes using the policy.
                proposal.status = self
                    .policy
                    .proposal_status(&proposal, self.ft_total_supply().0);
                if proposal.status == ProposalStatus::Approved {
                    self.internal_execute_proposal(&proposal);
                    true
                } else if proposal.status == ProposalStatus::Removed {
                    self.internal_reject_proposal(&proposal);
                    self.proposals.remove(&id);
                    false
                } else if proposal.status == ProposalStatus::Rejected {
                    self.internal_reject_proposal(&proposal);
                    true
                } else {
                    // Still in progress.
                    true
                }
            }
            Action::MoveToHub => false,
        };
        if update {
            self.proposals.insert(&id, &proposal);
        }
    }
}
