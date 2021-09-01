
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::env::*;
use near_sdk::json_types::Base64VecU8;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, AccountId, Balance, Gas};

/// Account ID used for $NEAR.
pub static BASE_TOKEN: &str = "base.token";

/// 1 yN to prevent access key fraud.
pub const ONE_YOCTO_NEAR: Balance = 1;

/// Gas for single ft_transfer call.
pub const GAS_FOR_FT_TRANSFER: Gas = Gas {
    0: 10_000_000_000_000,
};

/// Gas for upgrading this contract on promise creation + deploying new contract.
pub const GAS_FOR_UPGRADE_SELF_DEPLOY: Gas = Gas {
    0: 30_000_000_000_000,
};

pub const GAS_FOR_UPGRADE_REMOTE_DEPLOY: Gas = Gas {
    0: 10_000_000_000_000,
};

/// Configuration of the DAO.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Config {
    /// Name of the DAO.
    pub name: String,
    /// Purpose of this DAO.
    pub purpose: String,
    /// Generic metadata. Can be used by specific UI to store additional data.
    /// This is not used by anything in the contract.
    pub metadata: Base64VecU8,
}

#[cfg(test)]
impl Config {
    pub fn test_config() -> Self {
        Self {
            name: "Test".to_string(),
            purpose: "to test".to_string(),
            metadata: Base64VecU8(vec![]),
        }
    }
}

/// Set of possible action to take.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum Action {
    /// Action to add proposal. Used internally.
    AddProposal,
    /// Action to remove given proposal. Used for immediate deletion in special cases.
    RemoveProposal,
    /// Vote to approve given proposal or bounty.
    VoteApprove,
    /// Vote to reject given proposal or bounty.
    VoteReject,
    /// Vote to remove given proposal or bounty (because it's spam).
    VoteRemove,
    /// Finalize proposal, called when it's expired to return the funds
    /// (or in the future can be used for early proposal closure).
    Finalize,
    /// Move a proposal to the hub to shift into another DAO.
    MoveToHub,
}

impl Action {
    pub fn to_policy_label(&self) -> String {
        format!("{:?}", self)
    }
}

/// Self upgrade, optimizes gas by not loading into memory the code.
pub(crate) fn upgrade_self(hash: &[u8]) {
    let current_id = env::current_account_id();
    let method_name = "migrate";
    let attached_gas = env::prepaid_gas() - env::used_gas() - GAS_FOR_UPGRADE_SELF_DEPLOY;

    // Load input (wasm code).
    let code = storage_read(hash);
    // schedule a Promise tx to this same contract
    let promise_id = promise_batch_create(&current_id);
    // 1st item in the Tx: "deploy contract" (code is taken from variable)
    promise_batch_action_deploy_contract(promise_id, &code.unwrap());
    // 2nd item in the Tx: call this_contract.migrate() with remaining gas
    promise_batch_action_function_call(promise_id, method_name, &vec![], 0, attached_gas);
}

pub(crate) fn upgrade_remote(receiver_id: &AccountId, method_name: &str, hash: &[u8]) {
    // Load input into register 0.
    storage_read(hash);
    let promise_id = promise_batch_create(receiver_id);
    let attached_gas = env::prepaid_gas() - env::used_gas() - GAS_FOR_UPGRADE_REMOTE_DEPLOY;
    promise_batch_action_function_call(
        promise_id,
        method_name,
        &vec![],
        0,
        attached_gas,
    );
}
