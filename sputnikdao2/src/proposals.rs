use std::convert::TryInto;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::Base64VecU8;
use near_sdk::{AccountId, Balance, Gas, PromiseOrValue};

use crate::policy::UserInfo;
use crate::types::{
    ext_fungible_token, Action, Config, BASE_TOKEN, GAS_FOR_FT_TRANSFER, ONE_YOCTO_NEAR,
};
use crate::*;

/// Status of proposal.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub enum ProposalStatus {
    InProgress,
    Success,
    Reject,
    Removed,
}

/// Kinds of proposals, doing different action.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
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
    Upgrade {},
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

/// Proposal that are sent to this DAO.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
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
        }
    }
}

#[near_bindgen]
impl Contract {
    /// Executes given proposal and updates the contract's state.
    fn internal_execute_proposal(&mut self, proposal: Proposal) -> PromiseOrValue<()> {
        match proposal.kind {
            ProposalKind::ChangeConfig { config } => {
                self.config = config;
                PromiseOrValue::Value(())
            }
            ProposalKind::ChangePolicy { policy } => {
                self.policy = policy;
                PromiseOrValue::Value(())
            }
            ProposalKind::FunctionCall {
                receiver_id,
                method_name,
                args,
                deposit,
                gas,
            } => Promise::new(receiver_id)
                .function_call(method_name.into_bytes(), args.into(), deposit, gas)
                .into(),
            // TODO: implement upgrade
            ProposalKind::Upgrade {} => PromiseOrValue::Value(()),
            ProposalKind::Transfer {
                token_id,
                receiver_id,
                amount,
            } => {
                if token_id == env::current_account_id() {
                    self.token
                        .internal_withdraw(&env::current_account_id(), amount);
                    self.token.internal_deposit(&receiver_id, amount);
                    PromiseOrValue::Value(())
                } else if token_id == BASE_TOKEN {
                    Promise::new(receiver_id).transfer(amount).into()
                } else {
                    ext_fungible_token::ft_transfer(
                        receiver_id,
                        U128(amount),
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
                    .internal_deposit(&env::current_account_id(), amount);
                PromiseOrValue::Value(())
            }
            ProposalKind::Burn { amount } => {
                self.token
                    .internal_withdraw(&env::current_account_id(), amount);
                PromiseOrValue::Value(())
            }
        }
    }

    fn internal_user_info(&self) -> UserInfo {
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
        assert!(env::attached_deposit() >= self.config.bond);

        // 1. validate proposal.
        // TODO: ???

        // 2. check permission of caller to add proposal.
        let account_id = env::predecessor_account_id();
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

    /// Removes given proposal by id, if permissions allow.
    pub fn remove_proposal(&mut self, id: u64) {
        let proposal = self.proposals.get(&id).expect("ERR_NO_PROPOSAL");
        assert!(
            self.policy.can_execute_action(
                self.internal_user_info(),
                &proposal.kind,
                &Action::RemoveProposal
            ),
            "ERR_PERMISSION_DENIED"
        );
        self.proposals.remove(&id);
    }
}
