use sputnikdao2::{ProposalInput, ProposalKind, Bounty, BountyClaim};

use crate::utils::*;
use near_sdk_sim::{call, to_yocto, UserAccount, view};
use near_sdk::json_types::U128;
use near_sdk_sim::transaction::ExecutionStatus;
use near_sdk::serde::export::TryFrom;

mod utils;

// Helper functions

fn setup_addbounty_proposal() -> (UserAccount, Contract, UserAccount) {
    let (root, dao) = setup_dao();
    let bounty_hunter = root.create_user(user(1), to_yocto("1000"));
    add_proposal(
        &root,
        &dao,
        ProposalInput {
            description: "add bounty".to_string(),
            kind: ProposalKind::AddBounty {
                bounty: Bounty {
                    description: "".to_string(),
                    token: "".to_string(),
                    amount: U128(to_yocto("5")),
                    times: 1,
                    max_deadline: U64(1925376849430593581)
                }
            },
        },
    )
    .assert_success();

    // Vote the AddBounty proposal in
    vote(vec![&root], &dao, 0);

    // Bounty hunter claims bounty
    call!(
        bounty_hunter,
        dao.bounty_claim(0, U64::from(1725376849430593581)),
        deposit = to_yocto("1")
    )
    .assert_success();

    (root, dao, bounty_hunter)
}

// Tests

#[test]
fn test_bounty_claim_multiple() {
    let (_, dao, bounty_hunter) = setup_addbounty_proposal();

    // Tries to claim again and should get an error
    let double_claim_status = call!(
        bounty_hunter,
        dao.bounty_claim(0, U64::from(1725376849430593581)),
        deposit = to_yocto("1")
    ).status();
    match double_claim_status {
        ExecutionStatus::Failure(f) => {
            assert!(f.to_string().contains("ERR_BOUNTY_ALL_CLAIMED"), "Didn't receive expected failure message.");
        }
        _ => panic!("Expected failure when account tries to claim same bounty twice.")
    }
}

#[test]
fn test_bounty_withdraw_claim() {
    let (_, dao, bounty_hunter) = setup_addbounty_proposal();
    let mut bounty_num_claims = view!(dao.get_bounty_number_of_claims(0)).unwrap_json::<u32>();
    assert_eq!(bounty_num_claims, 1, "Bounty should have only one bounty claimed.");
    let mut bounty_hunter_claims = view!(dao.get_bounty_claims(ValidAccountId::try_from(user(1)).unwrap())).unwrap_json::<Vec<BountyClaim>>();
    assert_eq!(bounty_hunter_claims.len(), 1, "Claimant should only have one bounty claimed.");

    // User withdraws claim
    call!(
        bounty_hunter,
        dao.bounty_giveup(0)
    ).assert_success();

    // Both bounty and user should now have 0 claims
    bounty_num_claims = view!(dao.get_bounty_number_of_claims(0)).unwrap_json();
    assert_eq!(bounty_num_claims, 0, "Bounty should have zero claims.");
    bounty_hunter_claims = view!(dao.get_bounty_claims(ValidAccountId::try_from(user(1)).unwrap())).unwrap_json();
    assert_eq!(bounty_hunter_claims.len(), 0, "Claimant should now have zero claims.");
}
