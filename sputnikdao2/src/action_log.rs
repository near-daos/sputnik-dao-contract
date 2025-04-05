use std::collections::VecDeque;

use crate::types::Action;
use crate::*;
use near_sdk::AccountId;

const ACTION_LOG_SIZE: usize = 20;

#[derive(Clone)]
#[near(serializers=[borsh, json])]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
pub struct ActionLog {
    pub account_id: AccountId,
    pub proposal_id: u64,
    pub action: Action,
    pub block_height: u64,
}

#[derive(Clone)]
#[near(serializers=[borsh, json])]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
pub struct BlockLog {
    pub block_height: u64,
}

impl ActionLog {
    pub fn new(
        account_id: AccountId,
        proposal_id: u64,
        action: Action,
        block_height: u64,
    ) -> ActionLog {
        ActionLog {
            account_id,
            proposal_id,
            action,
            block_height,
        }
    }
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
        proposal: &mut ProposalV1,
    ) {
        update_action_log(
            &mut proposal.last_actions_log,
            BlockLog {
                block_height: env::block_height(),
            },
        );
        update_action_log(
            &mut self.actions_log,
            ActionLog {
                account_id: env::predecessor_account_id(),
                proposal_id,
                action,
                block_height: env::block_height(),
            },
        );
    }
}
