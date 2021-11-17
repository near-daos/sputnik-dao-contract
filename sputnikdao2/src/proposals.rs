use std::collections::HashMap;

use near_contract_standards::fungible_token::core_impl::ext_fungible_token;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{Base64VecU8, U128, U64};
use near_sdk::{ext_contract, log, AccountId, Balance, Gas, PromiseOrValue, PromiseResult};

use crate::policy::UserInfo;
use crate::types::{
    upgrade_remote, upgrade_self, Action, Config, GAS_FOR_COMMON_OPERATIONS, GAS_FOR_FT_TRANSFER,
    GAS_RESERVED_FOR_LATER, ONE_YOCTO_NEAR,
};
use crate::*;

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

/// Function call arguments.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct ActionCall {
    method_name: String,
    args: Base64VecU8,
    deposit: U128,
    gas: U64,
}

/// Kinds of proposals, doing different action.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
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
        #[serde(with = "serde_with::rust::string_empty_as_none")]
        token_id: Option<AccountId>,
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
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
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
    /// Count of votes per role per decision: yes / no / spam.
    pub vote_counts: HashMap<String, [Balance; 3]>,
    /// Map of who voted and how.
    pub votes: HashMap<AccountId, Vote>,
    /// Submission time (for voting period).
    pub submission_time: U64,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum VersionedProposal {
    Default(Proposal),
}

impl From<VersionedProposal> for Proposal {
    fn from(v: VersionedProposal) -> Self {
        match v {
            VersionedProposal::Default(p) => p,
        }
    }
}

impl Proposal {
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
            let amount = if policy.is_token_weighted(role, &self.kind.to_policy_label().to_string())
            {
                user_weight
            } else {
                1
            };
            self.vote_counts.entry(role.clone()).or_insert([0u128; 3])[vote.clone() as usize] +=
                amount;
        }
        assert!(
            self.votes.insert(account_id.clone(), vote).is_none(),
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
            vote_counts: HashMap::default(),
            votes: HashMap::default(),
            submission_time: U64::from(env::block_timestamp()),
        }
    }
}

// TODO: Use near_contract_standards::storage_management::StorageBalance.
// The original 'StorageBalance' struct does not support deserialization. Delete this
// and use the original one after https://github.com/near/near-sdk-rs/pull/630 gets released.
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct StorageBalance {
    pub total: U128,
    pub available: U128,
}

#[ext_contract(ext_storage_management)]
pub trait StorageManagement {
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance;
    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance>;
}

#[ext_contract(ext_self)]
pub trait ExtSelf {
    fn callback_after_storage_deposit(
        &mut self,
        token_id: AccountId,
        proposer_account: AccountId,
        receiver_id: AccountId,
        amount: U128,
        attached_deposit: U128,
        memo: String,
        msg: Option<String>,
    ) -> PromiseOrValue<()>;
    fn callback_after_storage_balance_of(
        &mut self,
        token_id: AccountId,
        proposer_account: AccountId,
        receiver_id: AccountId,
        amount: U128,
        attached_deposit: U128,
        memo: String,
        msg: Option<String>,
    ) -> PromiseOrValue<()>;
}

#[near_bindgen]
impl Contract {
    #[allow(dead_code)]
    #[private]
    pub fn callback_after_storage_deposit(
        &mut self,
        token_id: AccountId,
        proposer_account: AccountId,
        receiver_id: AccountId,
        amount: U128,
        attached_deposit: U128,
        memo: String,
        msg: Option<String>,
    ) -> PromiseOrValue<()> {
        assert_eq!(
            env::promise_results_count(),
            1,
            "ERR_UNEXPECTED_CALLBACK_PROMISES"
        );
        match env::promise_result(0) {
            PromiseResult::NotReady => {
                Promise::new(proposer_account.clone()).transfer(attached_deposit.0);
                panic!("storage_deposit Error: Received PromiseResult::NotReady");
            }
            PromiseResult::Failed => {
                Promise::new(proposer_account.clone()).transfer(attached_deposit.0);
                panic!("storage_deposit Error: Received PromiseResult::Failed");
            }
            PromiseResult::Successful(result) => {
                let balance = near_sdk::serde_json::from_slice::<StorageBalance>(&result).unwrap();

                // Pay back the proposal bond - registration fee.
                Promise::new(proposer_account.clone())
                    .transfer(attached_deposit.0 - balance.total.0);
                self.internal_payout(&token_id, &receiver_id, amount.0, memo, msg)
            }
        }
    }

    #[allow(dead_code)]
    #[private]
    pub fn callback_after_storage_balance_of(
        &mut self,
        token_id: AccountId,
        proposer_account: AccountId,
        receiver_id: AccountId,
        amount: U128,
        attached_deposit: U128,
        memo: String,
        msg: Option<String>,
    ) -> PromiseOrValue<()> {
        assert_eq!(
            env::promise_results_count(),
            1,
            "ERR_UNEXPECTED_CALLBACK_PROMISES"
        );
        match env::promise_result(0) {
            PromiseResult::NotReady => {
                Promise::new(proposer_account.clone()).transfer(attached_deposit.0);
                panic!("storage_balance_of Error: Received PromiseResult::NotReady");
            }
            PromiseResult::Failed => {
                Promise::new(proposer_account.clone()).transfer(attached_deposit.0);
                panic!("storage_balance_of Error: Received PromiseResult::Failed");
            }
            PromiseResult::Successful(result) => {
                let balance =
                    near_sdk::serde_json::from_slice::<Option<StorageBalance>>(&result).unwrap();

                if balance.is_some() {
                    // If the receiver account is already registered, pay back the proposal bond.
                    Promise::new(proposer_account.clone()).transfer(attached_deposit.0);
                    self.internal_payout(&token_id, &receiver_id, amount.0, memo, msg)
                } else {
                    // Otherwise, register the account and pay the registration fee from the proposal bond.
                    ext_storage_management::storage_deposit(
                        Some(receiver_id.clone()),
                        Some(true),
                        token_id.clone(),
                        attached_deposit.0,
                        GAS_FOR_COMMON_OPERATIONS,
                    )
                    .then(ext_self::callback_after_storage_deposit(
                        token_id.clone(),
                        proposer_account.clone(),
                        receiver_id.clone(),
                        amount,
                        attached_deposit,
                        memo,
                        msg,
                        env::current_account_id(),
                        0,
                        env::prepaid_gas()
                            - env::used_gas()
                            - GAS_FOR_COMMON_OPERATIONS
                            - GAS_RESERVED_FOR_LATER,
                    ))
                    .into()
                }
            }
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
        memo: String,
        msg: Option<String>,
    ) -> PromiseOrValue<()> {
        if let Some(msg) = msg {
            ext_fungible_token::ft_transfer_call(
                receiver_id.clone(),
                U128(amount),
                Some(memo),
                msg,
                token_id.clone(),
                ONE_YOCTO_NEAR,
                GAS_FOR_FT_TRANSFER,
            )
            .into()
        } else {
            ext_fungible_token::ft_transfer(
                receiver_id.clone(),
                U128(amount),
                Some(memo),
                token_id.clone(),
                ONE_YOCTO_NEAR,
                GAS_FOR_FT_TRANSFER,
            )
            .into()
        }
    }

    pub(crate) fn internal_try_register_and_payout(
        &mut self,
        token_id: &Option<AccountId>,
        proposer_account: &AccountId,
        receiver_id: &AccountId,
        amount: U128,
        attached_deposit: U128,
        memo: String,
        msg: Option<String>,
    ) -> PromiseOrValue<()> {
        if token_id.is_none() {
            Promise::new(proposer_account.clone()).transfer(attached_deposit.0);
            Promise::new(receiver_id.clone()).transfer(amount.0).into()
        } else {
            ext_storage_management::storage_balance_of(
                receiver_id.clone(),
                token_id.as_ref().unwrap().clone(),
                0,
                GAS_FOR_COMMON_OPERATIONS,
            )
            .then(ext_self::callback_after_storage_balance_of(
                token_id.as_ref().unwrap().clone(),
                proposer_account.clone(),
                receiver_id.clone(),
                amount,
                attached_deposit,
                memo,
                msg,
                env::current_account_id(),
                0,
                env::prepaid_gas()
                    - env::used_gas()
                    - GAS_FOR_COMMON_OPERATIONS
                    - GAS_RESERVED_FOR_LATER,
            ))
            .into()
        }
    }

    /// Executes given proposal and updates the contract's state.
    fn internal_execute_proposal(
        &mut self,
        policy: &Policy,
        proposal: &Proposal,
    ) -> PromiseOrValue<()> {
        // If it's not a transfer, return the proposal bond right away.
        // For a transfer, we might use the proposal bond to pay the deposit when registering the receiver account.
        if !matches!(&proposal.kind, ProposalKind::Transfer { .. })
            && !matches!(&proposal.kind, ProposalKind::BountyDone { .. })
        {
            // Return the proposal bond.
            Promise::new(proposal.proposer.clone()).transfer(policy.proposal_bond.0);
        }
        match &proposal.kind {
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
                        action.deposit.0,
                        Gas(action.gas.0),
                    )
                }
                promise.into()
            }
            ProposalKind::UpgradeSelf { hash } => {
                upgrade_self(&CryptoHash::from(hash.clone()));
                PromiseOrValue::Value(())
            }
            ProposalKind::UpgradeRemote {
                receiver_id,
                method_name,
                hash,
            } => {
                upgrade_remote(
                    &receiver_id.clone().into(),
                    method_name,
                    &CryptoHash::from(hash.clone()),
                );
                PromiseOrValue::Value(())
            }
            ProposalKind::Transfer {
                token_id,
                receiver_id,
                amount,
                msg,
            } => self.internal_try_register_and_payout(
                &token_id,
                &proposal.proposer,
                &receiver_id.clone().into(),
                amount.clone(),
                policy.proposal_bond.clone(),
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
            } => self.internal_execute_bounty_payout(
                &policy,
                &proposal,
                *bounty_id,
                &receiver_id.clone().into(),
                true,
            ),
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
            } => self.internal_execute_bounty_payout(
                &policy,
                &proposal,
                *bounty_id,
                &receiver_id.clone().into(),
                false,
            ),
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

#[near_bindgen]
impl Contract {
    /// Add proposal to this DAO.
    #[payable]
    pub fn add_proposal(&mut self, proposal: ProposalInput) -> u64 {
        // 0. validate bond attached.
        // TODO: consider bond in the token of this DAO.
        let policy = self.policy.get().unwrap().to_policy();
        assert!(
            env::attached_deposit() >= policy.proposal_bond.0,
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
                    !(token_id.is_none()) || msg.is_none(),
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
            .insert(&id, &VersionedProposal::Default(proposal.into()));
        self.last_proposal_id += 1;
        id
    }

    /// Act on given proposal by id, if permissions allow.
    /// Memo is logged but not stored in the state. Can be used to leave notes or explain the action.
    pub fn act_proposal(&mut self, id: u64, action: Action, memo: Option<String>) {
        let mut proposal: Proposal = self.proposals.get(&id).expect("ERR_NO_PROPOSAL").into();
        let policy = self.policy.get().unwrap().to_policy();
        // Check permissions for the given action.
        let (roles, allowed) =
            policy.can_execute_action(self.internal_user_info(), &proposal.kind, &action);
        assert!(allowed, "ERR_PERMISSION_DENIED");
        let sender_id = env::predecessor_account_id();
        // Update proposal given action. Returns true if should be updated in storage.
        let update = match action {
            Action::AddProposal => env::panic_str("ERR_WRONG_ACTION"),
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
                proposal.update_votes(
                    &sender_id,
                    &roles,
                    Vote::from(action),
                    &policy,
                    self.get_user_weight(&sender_id),
                );
                // Updates proposal status with new votes using the policy.
                proposal.status =
                    policy.proposal_status(&proposal, roles, self.total_delegation_amount);
                if proposal.status == ProposalStatus::Approved {
                    self.internal_execute_proposal(&policy, &proposal);
                    true
                } else if proposal.status == ProposalStatus::Removed {
                    self.internal_reject_proposal(&policy, &proposal, false);
                    self.proposals.remove(&id);
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
                proposal.status = policy.proposal_status(
                    &proposal,
                    policy.roles.iter().map(|r| r.name.clone()).collect(),
                    self.total_delegation_amount,
                );
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
            self.proposals
                .insert(&id, &VersionedProposal::Default(proposal));
        }
        if let Some(memo) = memo {
            log!("Memo: {}", memo);
        }
    }
}
