use crate::*;
use std::cmp::min;

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
}
