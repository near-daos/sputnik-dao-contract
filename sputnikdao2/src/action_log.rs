use std::collections::VecDeque;

use crate::types::Action;
use crate::*;
use near_sdk::json_types::U64;
use near_sdk::AccountId;

const ACTION_LOG_SIZE: usize = 20;

#[derive(Clone)]
#[near(serializers=[borsh, json])]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
pub struct ActionLog {
    pub account_id: AccountId,
    pub proposal_id: U64,
    pub action: Action,
    pub block_height: U64,
}

#[derive(Clone)]
#[near(serializers=[borsh, json])]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
pub struct ProposalLog {
    pub block_height: U64,
}

fn update_action_log<T>(log: &mut VecDeque<T>, action: T) {
    if log.len() >= ACTION_LOG_SIZE {
        log.pop_front(); // Remove oldest element when full
    }
    log.push_back(action);
}

#[near]
impl Contract {
    pub(crate) fn internal_log_action(
        &mut self,
        proposal_id: u64,
        action: Action,
        proposal: &mut Proposal,
    ) {
        update_action_log(
            &mut proposal.last_actions_log,
            ProposalLog {
                block_height: env::block_height().into(),
            },
        );
        update_action_log(
            &mut self.actions_log,
            ActionLog {
                account_id: env::predecessor_account_id(),
                proposal_id: proposal_id.into(),
                action,
                block_height: env::block_height().into(),
            },
        );
    }
}
