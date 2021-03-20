use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{Base64VecU8, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{ext_contract, serde_json, Balance, Gas};

/// Account ID used for $NEAR.
pub const BASE_TOKEN: &str = "";

/// 1 yN to prevent access key fraud.
pub const ONE_YOCTO_NEAR: Balance = 1;

/// Gas for single ft_transfer call.
pub const GAS_FOR_FT_TRANSFER: Gas = 10_000_000_000_000;

/// No deposit.
pub const NO_DEPOSIT: Balance = 0;

/// Gas for upgrading remote contract on promise creation.
pub const GAS_FOR_UPGRADE_REMOTE_PROMISE: Gas = 10_000_000_000_000;

/// Gas for upgrading this contract on promise creation + deploying new contract.
pub const GAS_FOR_UPGRADE_SELF_DEPLOY: Gas = 10_000_000_000_000;

/// Configuration of the DAO.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Config {
    pub name: String,
    pub symbol: String,
    pub icon: Option<String>,
    pub reference: Option<String>,
    pub reference_hash: Option<Base64VecU8>,
    pub decimals: u8,
    pub purpose: String,
    pub bond: U128,
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
    /// Vote to remove given proposal or bounty (because it's spam)
    VoteRemove,
    /// Move a proposal to the hub to shift into another DAO.
    MoveToHub,
}

impl Action {
    pub fn to_policy_label(&self) -> String {
        serde_json::to_string(&self).expect("Must serialize")
    }
}
