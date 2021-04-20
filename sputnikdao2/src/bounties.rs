use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{WrappedDuration, WrappedTimestamp, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, Promise, PromiseOrValue};

use crate::*;

/// Information recorded about claim of the bounty by given user.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct BountyClaim {
    /// Bounty id that was claimed.
    bounty_id: u64,
    /// Start time of the claim.
    start_time: WrappedTimestamp,
    /// Deadline specified by claimer.
    deadline: WrappedDuration,
    /// Completed?
    completed: bool,
}

/// Bounty information.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Bounty {
    /// Description of the bounty.
    pub description: String,
    /// Token the bounty will be paid out.
    pub token: AccountId,
    /// Amount to be paid out.
    pub amount: U128,
    /// How many times this bounty can be done.
    pub times: u32,
    /// Max deadline from claim that can be spend on this bounty.
    pub max_deadline: WrappedDuration,
}

impl Contract {
    /// Adds bounty to the storage and returns it's id.
    /// Must not fail.
    pub(crate) fn internal_add_bounty(&mut self, bounty: Bounty) -> u64 {
        let id = self.data().last_bounty_id;
        self.data_mut().bounties.insert(&id, &bounty.into());
        self.data_mut().last_bounty_id += 1;
        id
    }

    /// This must be called when proposal to payout bounty has been voted either successfully or not.
    pub(crate) fn internal_execute_bounty_payout(
        &mut self,
        id: u64,
        receiver_id: &AccountId,
        success: bool,
    ) -> PromiseOrValue<()> {
        let mut bounty = self.data_mut().bounties.get(&id).expect("ERR_NO_BOUNTY");
        let (claims, claim_idx) = self.internal_get_claims(id, &receiver_id);
        self.internal_remove_claim(id, claims, claim_idx);
        if success {
            if bounty.times == 0 {
                self.data_mut().bounties.remove(&id);
            } else {
                bounty.times -= 1;
                self.data_mut().bounties.insert(&id, &bounty);
            }
            self.internal_payout(&bounty.token, receiver_id, bounty.amount.0)
        } else {
            PromiseOrValue::Value(())
        }
    }

    fn internal_find_claim(&self, bounty_id: u64, claims: &[BountyClaim]) -> Option<usize> {
        for i in 0..claims.len() {
            if claims[i].bounty_id == bounty_id {
                return Some(i);
            }
        }
        None
    }
}

#[near_bindgen]
impl Contract {
    /// Claim given bounty by caller with given expected duration to execute.
    /// Bond must be attached to the claim.
    /// Fails if already claimed `repeat` times.
    #[payable]
    pub fn bounty_claim(&mut self, id: u64, deadline: WrappedDuration) {
        let bounty = self.data_mut().bounties.get(&id).expect("ERR_NO_BOUNTY");
        let policy = self.data_mut().policy.get().unwrap().to_policy();
        assert_eq!(
            env::attached_deposit(),
            policy.bounty_bond.0,
            "ERR_BOUNTY_WRONG_BOND"
        );
        let claims_count = self
            .data_mut()
            .bounty_claims_count
            .get(&id)
            .unwrap_or_default();
        assert!(claims_count < bounty.times, "ERR_BOUNTY_ALL_CLAIMED");
        assert!(
            deadline.0 <= bounty.max_deadline.0,
            "ERR_BOUNTY_WRONG_DEADLINE"
        );
        self.data_mut()
            .bounty_claims_count
            .insert(&id, &(claims_count + 1));
        let mut claims = self
            .data_mut()
            .bounty_claimers
            .get(&env::predecessor_account_id())
            .unwrap_or_default();
        claims.push(BountyClaim {
            bounty_id: id,
            start_time: WrappedTimestamp::from(env::block_timestamp()),
            deadline,
            completed: false,
        });
        self.data_mut()
            .bounty_claimers
            .insert(&env::predecessor_account_id(), &claims);
    }

    /// Removes given claims from this bounty and user's claims.
    fn internal_remove_claim(&mut self, id: u64, mut claims: Vec<BountyClaim>, claim_idx: usize) {
        claims.remove(claim_idx);
        if claims.len() == 0 {
            self.data_mut()
                .bounty_claimers
                .remove(&env::predecessor_account_id());
        } else {
            self.data_mut()
                .bounty_claimers
                .insert(&env::predecessor_account_id(), &claims);
        }
        let count = self.data().bounty_claims_count.get(&id).unwrap() - 1;
        self.data_mut().bounty_claims_count.insert(&id, &count);
    }

    fn internal_get_claims(&mut self, id: u64, sender_id: &AccountId) -> (Vec<BountyClaim>, usize) {
        let claims = self
            .data_mut()
            .bounty_claimers
            .get(&sender_id)
            .expect("ERR_NO_BOUNTY_CLAIMS");
        let claim_idx = self
            .internal_find_claim(id, &claims)
            .expect("ERR_NO_BOUNTY_CLAIM");
        (claims, claim_idx)
    }

    /// Report that bounty is done. Creates a proposal to vote for paying out the bounty.
    /// Only creator of the claim can call `done` on bounty that is still in progress.
    /// On expired, anyone can call it to free up the claim slot.
    pub fn bounty_done(&mut self, id: u64, account_id: Option<AccountId>) {
        let sender_id = account_id.unwrap_or_else(|| env::predecessor_account_id());
        let (mut claims, claim_idx) = self.internal_get_claims(id, &sender_id);
        assert!(!claims[claim_idx].completed, "ERR_BOUNTY_CLAIM_COMPLETED");
        if env::block_timestamp() > claims[claim_idx].start_time.0 + claims[claim_idx].deadline.0 {
            // Expired. Nothing to do.
            self.internal_remove_claim(id, claims, claim_idx);
        } else {
            // Still under deadline. Only the user themself can call this.
            assert_eq!(
                sender_id,
                env::predecessor_account_id(),
                "ERR_BOUNTY_DONE_MUST_BE_SELF"
            );
            self.add_proposal(ProposalInput {
                description: format!("Bounty {} done", id),
                kind: ProposalKind::BountyDone {
                    bounty_id: id,
                    receiver_id: sender_id.clone(),
                },
            });
            claims[claim_idx].completed = true;
            self.data_mut().bounty_claimers.insert(&sender_id, &claims);
        }
    }

    /// Give up working on the bounty.
    pub fn bounty_giveup(&mut self, id: u64) -> PromiseOrValue<()> {
        let policy = self.data_mut().policy.get().unwrap().to_policy();
        let (claims, claim_idx) = self.internal_get_claims(id, &env::predecessor_account_id());
        let result = if env::block_timestamp() - claims[claim_idx].start_time.0
            > policy.bounty_forgiveness_period.0
        {
            // If user over the forgiveness period.
            PromiseOrValue::Value(())
        } else {
            // Within forgiveness period.
            Promise::new(env::predecessor_account_id())
                .transfer(policy.bounty_bond.0)
                .into()
        };
        self.internal_remove_claim(id, claims, claim_idx);
        result
    }
}

#[cfg(test)]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{testing_env, MockedBlockchain};
    use near_sdk_sim::to_yocto;

    use crate::proposals::{ProposalInput, ProposalKind};
    use crate::types::BASE_TOKEN;
    use crate::{Action, Config};

    use super::*;

    /// Adds a bounty, and tests it's full lifecycle.
    #[test]
    fn test_bounty_lifecycle() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        let mut contract = Contract::new(
            Config::test_config(),
            VersionedPolicy::Default(vec![accounts(1).into()]),
        );
        testing_env!(context.attached_deposit(to_yocto("1")).build());
        contract.add_proposal(ProposalInput {
            description: "test".to_string(),
            kind: ProposalKind::AddBounty {
                bounty: Bounty {
                    description: "test bounty".to_string(),
                    token: BASE_TOKEN.to_string(),
                    amount: U128(to_yocto("10")),
                    times: 2,
                    max_deadline: WrappedDuration::from(1_000),
                },
            },
        });
        assert_eq!(contract.get_last_bounty_id(), 0);

        contract.act_proposal(0, Action::VoteApprove);

        assert_eq!(contract.get_last_bounty_id(), 1);
        assert_eq!(contract.get_bounty(0).bounty.times, 2);

        contract.bounty_claim(0, WrappedDuration::from(500));
        assert_eq!(contract.get_bounty_claims(accounts(1)).len(), 1);
        assert_eq!(contract.get_bounty_number_of_claims(0), 1);

        contract.bounty_giveup(0);
        assert_eq!(contract.get_bounty_claims(accounts(1)).len(), 0);
        assert_eq!(contract.get_bounty_number_of_claims(0), 0);

        contract.bounty_claim(0, WrappedDuration::from(500));
        assert_eq!(contract.get_bounty_claims(accounts(1)).len(), 1);
        assert_eq!(contract.get_bounty_number_of_claims(0), 1);

        contract.bounty_done(0, None);
        assert!(contract.get_bounty_claims(accounts(1))[0].completed);

        assert_eq!(contract.get_last_proposal_id(), 2);
        assert_eq!(
            contract.get_proposal(1).proposal.kind.to_policy_label(),
            "bounty_done"
        );

        contract.act_proposal(1, Action::VoteApprove);

        assert_eq!(contract.get_bounty_claims(accounts(1)).len(), 0);
        assert_eq!(contract.get_bounty(0).bounty.times, 1);
    }
}
