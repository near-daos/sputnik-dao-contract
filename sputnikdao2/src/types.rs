use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{Base64VecU8, U128};
use near_sdk::serde::{self, Deserialize, Serialize};
use near_sdk::{ext_contract, serde_json, AccountId, Balance, Gas};

/// Account ID used for $NEAR.
pub const BASE_TOKEN: &str = "";

/// 1 yN to prevent access key fraud.
pub const ONE_YOCTO_NEAR: Balance = 1;

/// Gas for single ft_transfer call.
pub const GAS_FOR_FT_TRANSFER: Gas = 10_000_000_000_000;

/// Configuration of the DAO.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Config {
    pub name: String,
    pub purpose: String,
    pub bond: Balance,
    pub symbol: String,
    pub decimals: u8,
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
    AddProposal,
    RemoveProposal,
    VoteYes,
    VoteNo,
    MoveProposalToHub,
}

impl Action {
    pub fn to_policy_label(&self) -> String {
        serde_json::to_string(&self).expect("Must serialize")
    }
}
