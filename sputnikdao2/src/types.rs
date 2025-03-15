use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::Base64VecU8;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, Balance, Gas};

/// Account ID used for $NEAR in near-sdk v3.
/// Need to keep it around for backward compatibility.
pub const OLD_BASE_TOKEN: &str = "";

/// Account ID that represents a token in near-sdk v3.
/// Need to keep it around for backward compatibility.
pub type OldAccountId = String;

/// 1 yN to prevent access key fraud.
pub const ONE_YOCTO_NEAR: Balance = 1;

/// Gas for single ft_transfer call.
pub const GAS_FOR_FT_TRANSFER: Gas = Gas(10_000_000_000_000);

/// Configuration of the DAO.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug, PartialEq)]
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

/// In near-sdk v3, the token was represented by a String, with no other restrictions.
/// That being said, Sputnik used "" (empty String) as a convention to represent the $NEAR token.
/// In near-sdk v4, the token representation was replaced by AccountId (which is in fact a wrapper
/// over a String), with the restriction that the token must be between 2 and 64 chars.
/// Sputnik had to adapt since "" was not allowed anymore and we chose to represent the token as a
/// Option<AccountId> with the convention that None represents the $NEAR token.
/// This function is required to help with the transition and keep the backward compatibility.
pub fn convert_old_to_new_token(old_account_id: &OldAccountId) -> Option<AccountId> {
    if old_account_id == OLD_BASE_TOKEN {
        return None;
    }
    Some(AccountId::new_unchecked(old_account_id.clone()))
}
