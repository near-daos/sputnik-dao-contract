use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::{U128, U64};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, Promise, PromiseOrValue};

use crate::types::{convert_old_to_new_token, OldAccountId};
use crate::*;

/// Information recorded about claim of the bounty by given user.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct BountyClaim {
    /// Bounty id that was claimed.
    bounty_id: u64,
    /// Start time of the claim.
    start_time: U64,
    /// Deadline specified by claimer.
    deadline: U64,
    /// Completed?
    completed: bool,
}

/// Bounty information.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
#[serde(crate = "near_sdk::serde")]
pub struct Bounty {
    /// Description of the bounty.
    pub description: String,
    /// Token the bounty will be paid out.
    /// Can be "" for $NEAR or a valid account id.
    pub token: OldAccountId,
    /// Amount to be paid out.
    pub amount: U128,
    /// How many times this bounty can be done.
    pub times: u32,
    /// Max deadline from claim that can be spend on this bounty.
    pub max_deadline: U64,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum VersionedBounty {
    Default(Bounty),
}

impl From<VersionedBounty> for Bounty {
    fn from(v: VersionedBounty) -> Self {
        match v {
            VersionedBounty::Default(b) => b,
        }
    }
}

impl Contract {
    /// Adds bounty to the storage and returns it's id.
    /// Must not fail.
    pub(crate) fn internal_add_bounty(&mut self, bounty: &Bounty) -> u64 {
        let id = self.last_bounty_id;
        self.bounties
            .insert(&id, &VersionedBounty::Default(bounty.clone()));
        self.last_bounty_id += 1;
        id
    }

    /// This must be called when proposal to payout bounty has been voted either successfully or not.
    pub(crate) fn internal_execute_bounty_payout(
        &mut self,
        id: u64,
        receiver_id: &AccountId,
        success: bool,
    ) -> PromiseOrValue<()> {
        let bounty: Bounty = self.bounties.get(&id).expect("ERR_NO_BOUNTY").into();
        self.internal_remove_claim(id, receiver_id);
        if success {
            self.internal_payout(
                &convert_old_to_new_token(&bounty.token),
                receiver_id,
                bounty.amount.0,
                format!("Bounty {} payout", id),
                None,
            )
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
    /// Fails if already claimed `times` times.
    #[payable]
    pub fn bounty_claim(&mut self, id: u64, deadline: U64) {
        let bounty: Bounty = self.bounties.get(&id).expect("ERR_NO_BOUNTY").into();
        let policy = self.policy.get().unwrap().to_policy();
        assert_eq!(
            env::attached_deposit(),
            policy.bounty_bond.0,
            "ERR_BOUNTY_WRONG_BOND"
        );
        let claims_count = self.bounty_claims_count.get(&id).unwrap_or_default();
        assert!(claims_count < bounty.times, "ERR_BOUNTY_ALL_CLAIMED");
        assert!(
            deadline.0 <= bounty.max_deadline.0,
            "ERR_BOUNTY_WRONG_DEADLINE"
        );
        self.bounty_claims_count.insert(&id, &(claims_count + 1));
        let mut claims = self
            .bounty_claimers
            .get(&env::predecessor_account_id())
            .unwrap_or_default();
        claims.push(BountyClaim {
            bounty_id: id,
            start_time: U64::from(env::block_timestamp()),
            deadline,
            completed: false,
        });
        self.bounty_claimers
            .insert(&env::predecessor_account_id(), &claims);
        self.locked_amount += env::attached_deposit();
    }

    /// Remove the claim of `claimer_id` from this bounty.
    fn internal_remove_claim(&mut self, bounty_id: u64, claimer_id: &AccountId) {
        let (mut claims, claim_idx) = self.internal_get_claims(bounty_id, claimer_id);
        claims.remove(claim_idx);
        if claims.len() == 0 {
            self.bounty_claimers.remove(claimer_id);
        } else {
            self.bounty_claimers.insert(claimer_id, &claims);
        }
        let count = self.bounty_claims_count.get(&bounty_id).unwrap() - 1;
        self.bounty_claims_count.insert(&bounty_id, &count);
    }

    fn internal_get_claims(&mut self, id: u64, sender_id: &AccountId) -> (Vec<BountyClaim>, usize) {
        let claims = self
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
    #[payable]
    pub fn bounty_done(&mut self, id: u64, account_id: Option<AccountId>, description: String) {
        let sender_id = account_id.unwrap_or_else(|| env::predecessor_account_id());
        let (mut claims, claim_idx) = self.internal_get_claims(id, &sender_id);
        assert!(!claims[claim_idx].completed, "ERR_BOUNTY_CLAIM_COMPLETED");
        if env::block_timestamp() > claims[claim_idx].start_time.0 + claims[claim_idx].deadline.0 {
            // Expired. Nothing to do.
            self.internal_remove_claim(id, &sender_id);
        } else {
            // Still under deadline. Only the user themself can call this.
            assert_eq!(
                sender_id,
                env::predecessor_account_id(),
                "ERR_BOUNTY_DONE_MUST_BE_SELF"
            );
            self.add_proposal(ProposalInput {
                description,
                kind: ProposalKind::BountyDone {
                    bounty_id: id,
                    receiver_id: sender_id.clone(),
                },
            });
            claims[claim_idx].completed = true;
            self.bounty_claimers.insert(&sender_id, &claims);
        }
    }

    /// Give up working on the bounty.
    pub fn bounty_giveup(&mut self, id: u64) -> PromiseOrValue<()> {
        let policy = self.policy.get().unwrap().to_policy();
        let (claims, claim_idx) = self.internal_get_claims(id, &env::predecessor_account_id());
        let result = if env::block_timestamp() - claims[claim_idx].start_time.0
            > policy.bounty_forgiveness_period.0
        {
            // If user over the forgiveness period.
            PromiseOrValue::Value(())
        } else {
            // Within forgiveness period. Return bond.
            self.locked_amount -= policy.bounty_bond.0;
            Promise::new(env::predecessor_account_id())
                .transfer(policy.bounty_bond.0)
                .into()
        };
        self.internal_remove_claim(id, &env::predecessor_account_id());
        result
    }
}

#[cfg(test)]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;
    use near_sdk_sim::to_yocto;

    use crate::proposals::{ProposalInput, ProposalKind};
    use crate::{Action, Config};

    use super::*;

    fn add_bounty(context: &mut VMContextBuilder, contract: &mut Contract, times: u32) -> u64 {
        testing_env!(context.attached_deposit(to_yocto("1")).build());
        let id = contract.add_proposal(ProposalInput {
            description: "test".to_string(),
            kind: ProposalKind::AddBounty {
                bounty: Bounty {
                    description: "test bounty".to_string(),
                    token: String::from(OLD_BASE_TOKEN),
                    amount: U128(to_yocto("10")),
                    times,
                    max_deadline: U64::from(1_000),
                },
            },
        });
        assert_eq!(contract.get_last_bounty_id(), id);
        contract.act_proposal(id, Action::VoteApprove, None);
        id
    }

    /// Adds a bounty, and tests it's full lifecycle.
    #[test]
    fn test_bounty_lifecycle() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        let mut contract = Contract::new(
            Config::test_config(),
            VersionedPolicy::Default(vec![accounts(1).into()]),
        );
        add_bounty(&mut context, &mut contract, 2);

        assert_eq!(contract.get_last_bounty_id(), 1);
        assert_eq!(contract.get_bounty(0).bounty.times, 2);

        contract.bounty_claim(0, U64::from(500));
        assert_eq!(contract.get_bounty_claims(accounts(1)).len(), 1);
        assert_eq!(contract.get_bounty_number_of_claims(0), 1);

        contract.bounty_giveup(0);
        assert_eq!(contract.get_bounty_claims(accounts(1)).len(), 0);
        assert_eq!(contract.get_bounty_number_of_claims(0), 0);

        contract.bounty_claim(0, U64::from(500));
        assert_eq!(contract.get_bounty_claims(accounts(1)).len(), 1);
        assert_eq!(contract.get_bounty_number_of_claims(0), 1);

        contract.bounty_done(0, None, "Bounty is done".to_string());
        assert!(contract.get_bounty_claims(accounts(1))[0].completed);

        assert_eq!(contract.get_last_proposal_id(), 2);
        assert_eq!(
            contract.get_proposal(1).proposal.kind.to_policy_label(),
            "bounty_done"
        );

        contract.act_proposal(1, Action::VoteApprove, None);
        testing_env!(
            context.build(),
            near_sdk::VMConfig::test(),
            near_sdk::RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(vec![])],
        );
        contract.on_proposal_callback(1);

        assert_eq!(contract.get_bounty_claims(accounts(1)).len(), 0);
        assert_eq!(contract.get_bounty(0).bounty.times, 1);

        contract.bounty_claim(0, U64::from(500));
        contract.bounty_done(0, None, "Bounty is done 2".to_string());
        contract.act_proposal(2, Action::VoteApprove, None);
        testing_env!(
            context.build(),
            near_sdk::VMConfig::test(),
            near_sdk::RuntimeFeesConfig::test(),
            Default::default(),
            vec![PromiseResult::Successful(vec![])],
        );
        contract.on_proposal_callback(2);

        assert_eq!(contract.get_bounty(0).bounty.times, 0);
    }

    #[test]
    #[should_panic(expected = "ERR_BOUNTY_ALL_CLAIMED")]
    fn test_bounty_claim_not_allowed() {
        let mut context = VMContextBuilder::new();
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        let mut contract = Contract::new(
            Config::test_config(),
            VersionedPolicy::Default(vec![accounts(1).into()]),
        );
        let id = add_bounty(&mut context, &mut contract, 1);
        contract.bounty_claim(id, U64::from(500));
        contract.bounty_done(id, None, "Bounty is done 2".to_string());
        contract.bounty_claim(id, U64::from(500));
    }
}
