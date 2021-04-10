use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::env::BLOCKCHAIN_INTERFACE;
use near_sdk::json_types::{Base64VecU8, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, ext_contract, serde_json, Balance, Gas};

const BLOCKCHAIN_INTERFACE_NOT_SET_ERR: &str = "Blockchain interface not set.";

/// Account ID used for $NEAR.
pub const BASE_TOKEN: &str = "";

/// 1 yN to prevent access key fraud.
pub const ONE_YOCTO_NEAR: Balance = 1;

/// Gas for single ft_transfer call.
pub const GAS_FOR_FT_TRANSFER: Gas = 10_000_000_000_000;

/// No deposit.
pub const NO_DEPOSIT: Balance = 0;

/// Gas for upgrading remote contract on promise creation.
pub const GAS_FOR_UPGRADE_REMOTE_PROMISE: Gas = 30_000_000_000_000;

/// Gas for upgrading this contract on promise creation + deploying new contract.
pub const GAS_FOR_UPGRADE_SELF_DEPLOY: Gas = 30_000_000_000_000;

/// Configuration of the DAO.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Config {
    /// Name of the DAO and of the token.
    pub name: String,
    /// Token metadata: symbol.
    pub symbol: String,
    /// Token metadata: Url for icon.
    pub icon: Option<String>,
    /// Token metadata: link to reference.
    pub reference: Option<String>,
    /// Token metadata: reference hash to validate that reference link fetches correct data.
    pub reference_hash: Option<Base64VecU8>,
    /// Number of decimals in the token.
    pub decimals: u8,
    /// Purpose of this DAO.
    pub purpose: String,
    /// Minimal bond attached with proposal.
    pub bond: U128,
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
            icon: None,
            reference: None,
            reference_hash: None,
            decimals: 24,
            bond: U128(10u128.pow(24)),
            symbol: "TEST".to_string(),
            metadata: Base64VecU8(vec![]),
        }
    }
}

/// External interface for Fungible tokens.
#[ext_contract(ext_fungible_token)]
pub trait FungibleTokenExt {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
}

/// Set of possible action to take.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
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
        serde_json::to_string(&self).expect("Must serialize")
    }
}

/// Self upgrade, optimizes gas by not loading into memory the code.
pub(crate) fn upgrade_self(hash: &[u8]) {
    let current_id = env::current_account_id().into_bytes();
    let method_name = "migrate".as_bytes().to_vec();
    let attached_gas = env::prepaid_gas() - env::used_gas() - GAS_FOR_UPGRADE_SELF_DEPLOY;
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
                .promise_batch_create(current_id.len() as _, current_id.as_ptr() as _);
            b.borrow()
                .as_ref()
                .expect(BLOCKCHAIN_INTERFACE_NOT_SET_ERR)
                .promise_batch_action_deploy_contract(promise_id, u64::MAX as _, 0);
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
