use std::collections::VecDeque;

use crate::types::Action;
use crate::*;
use near_sdk::AccountId;

const ACTION_LOG_SIZE: usize = 20;

#[derive(Clone)]
#[near(serializers=[borsh, json])]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, PartialEq))]
pub struct ActionLog {
    account_id: AccountId,
    proposal_id: u64,
    action: Action,
    block_height: u64,
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

fn update_action_log(log: &mut VecDeque<ActionLog>, action: ActionLog) {
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
        let new_action = ActionLog {
            account_id: env::predecessor_account_id(),
            proposal_id,
            action,
            block_height: env::block_height(),
        };
        update_action_log(
            proposal.last_actions_log.as_mut().unwrap(),
            new_action.clone(),
        );
        update_action_log(&mut self.actions_log, new_action);
    }
}
