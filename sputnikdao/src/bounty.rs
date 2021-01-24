use std::collections::HashSet;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
enum BountyStatus {
    Open,
    /// When was bounty claimed and by whom.
    Claimed { account_id: AccountId, started: Timestamp },
    /// Bounty is done by given account, review started at given time.
    InReview { account_id: AccountId, started: Timestamp },
    /// Bounty is done and closed.
    Done,
    /// Review expired, will pay out.
    Expired,
}

/// Stores information about bounties that this DAO has open.
/// Bounty can be `Open`, `InProgress`, `Ready`
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Bounty {
    /// Status of the given bounty.
    status: BountyStatus,
    description: String,
    /// Maximum how long should this bounty take.
    duration: Duration,
    /// Applicants.
    /// TODO: add details and their proposed duration?
    applicants: HashSet<AccountId>,
}
