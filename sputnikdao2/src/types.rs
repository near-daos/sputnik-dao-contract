use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::env::BLOCKCHAIN_INTERFACE;
use near_sdk::json_types::Base64VecU8;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, AccountId, Balance, Gas};

const BLOCKCHAIN_INTERFACE_NOT_SET_ERR: &str = "Blockchain interface not set.";

/// Account ID used for $NEAR.
pub const BASE_TOKEN: &str = "";

/// 1 yN to prevent access key fraud.
pub const ONE_YOCTO_NEAR: Balance = 1;

/// Gas for single ft_transfer call.
pub const GAS_FOR_FT_TRANSFER: Gas = 10_000_000_000_000;

/// Gas for upgrading this contract on promise creation + deploying new contract.
pub const GAS_FOR_UPGRADE_SELF_DEPLOY: Gas = 30_000_000_000_000;

pub const GAS_FOR_UPGRADE_REMOTE_DEPLOY: Gas = 10_000_000_000_000;

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

/// Set of possible actions that a user may be allowed to take on
/// proposals.
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
    let current_id = env::current_account_id().into_bytes();
    let method_name = "migrate".as_bytes().to_vec();
    let attached_gas = env::prepaid_gas() - env::used_gas() - GAS_FOR_UPGRADE_SELF_DEPLOY;
    unsafe {
        BLOCKCHAIN_INTERFACE.with(|b| {
            // Load input (wasm code) into register 0.
            b.borrow()
                .as_ref()
                .expect(BLOCKCHAIN_INTERFACE_NOT_SET_ERR)
                .storage_read(hash.len() as _, hash.as_ptr() as _, 0);
            // schedule a Promise tx to this same contract
            let promise_id = b
                .borrow()
                .as_ref()
                .expect(BLOCKCHAIN_INTERFACE_NOT_SET_ERR)
                .promise_batch_create(current_id.len() as _, current_id.as_ptr() as _);
            // 1st item in the Tx: "deploy contract" (code is taken from register 0)
            b.borrow()
                .as_ref()
                .expect(BLOCKCHAIN_INTERFACE_NOT_SET_ERR)
                .promise_batch_action_deploy_contract(promise_id, u64::MAX as _, 0);
            // 2nd item in the Tx: call this_contract.migrate() with remaining gas
            b.borrow()
                .as_ref()
                .expect(BLOCKCHAIN_INTERFACE_NOT_SET_ERR)
                .promise_batch_action_function_call(
                    promise_id,
                    method_name.len() as _,
                    method_name.as_ptr() as _,
                    0 as _,
                    0 as _,
                    0 as _,
                    attached_gas,
                );
        });
    }
}

pub(crate) fn upgrade_remote(receiver_id: &AccountId, method_name: &str, hash: &[u8]) {
    unsafe {
        BLOCKCHAIN_INTERFACE.with(|b| {
            // Load input into register 0.
            b.borrow()
                .as_ref()
                .expect(BLOCKCHAIN_INTERFACE_NOT_SET_ERR)
                .storage_read(hash.len() as _, hash.as_ptr() as _, 0);
            let promise_id = b
                .borrow()
                .as_ref()
                .expect(BLOCKCHAIN_INTERFACE_NOT_SET_ERR)
                .promise_batch_create(receiver_id.len() as _, receiver_id.as_ptr() as _);
            let attached_gas = env::prepaid_gas() - env::used_gas() - GAS_FOR_UPGRADE_REMOTE_DEPLOY;
            b.borrow()
                .as_ref()
                .expect(BLOCKCHAIN_INTERFACE_NOT_SET_ERR)
                .promise_batch_action_function_call(
                    promise_id,
                    method_name.len() as _,
                    method_name.as_ptr() as _,
                    u64::MAX as _,
                    0 as _,
                    0 as _,
                    attached_gas,
                );
        });
    }
}
