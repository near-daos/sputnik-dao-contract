use std::cmp::min;

use crate::*;

#[near_bindgen]
impl Contract {
    pub fn get_config(&self) -> Config {
        self.config.clone()
    }

    pub fn get_policy(&self) -> Policy {
        self.policy.clone()
    }

    pub fn get_last_proposal_id(&self) -> u64 {
        self.last_proposal_id
    }

    pub fn get_proposals(&self, from_index: u64, limit: u64) -> Vec<Proposal> {
        (from_index..min(self.last_proposal_id, from_index + limit))
            .filter_map(|id| self.proposals.get(&id))
            .collect()
    }

    pub fn get_proposal(&self, id: u64) -> Proposal {
        self.proposals.get(&id).expect("ERR_NO_PROPOSAL")
    }

    /// Get given bounty by id.
    pub fn get_bounty(&self, id: u64) -> Bounty {
        self.bounties.get(&id).expect("ERR_NO_BOUNTY")
    }

    /// Get number of bounties.
    pub fn get_last_bounty_id(&self) -> u64 {
        self.last_bounty_id
    }

    /// Get `limit` of bounties from given index.
    pub fn get_bounties(&self, from_index: u64, limit: u64) -> Vec<Bounty> {
        (from_index..std::cmp::min(from_index + limit, self.last_bounty_id))
            .filter_map(|index| self.bounties.get(&index))
            .collect()
    }

    /// Get bounty claims for given user.
    pub fn get_bounty_claims(&self, account_id: ValidAccountId) -> Vec<BountyClaim> {
        self.bounty_claimers
            .get(account_id.as_ref())
            .unwrap_or_default()
    }

    /// Returns number of claims per given bounty.
    pub fn get_bounty_number_of_claims(&self, id: u64) -> u32 {
        self.bounty_claims_count.get(&id).unwrap_or_default()
    }
}
