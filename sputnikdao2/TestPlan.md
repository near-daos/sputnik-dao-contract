# SputnikDao

## struct Contract

### add_proposal
1. Check that the proposal is added to the list of proposals
2. Check that ProposalInput can have any ProposalKind (or is it not required?)
3. Check that only those with a permission can add the proposal
4. Chech that the method fails in case of insufficient deposit 
### act_proposal
1. Check that only those with a permission can act on the the proposal
2. Check that the method works correctly on all possible Actions. Also shold act differently during expired or failed finalization

### migrate
1. Can only be called by the contract
2. Should migrate initial state
### remove_blob
1. Can only be called by the original storer
2. Blob is removed
3. The payback is computed correctly

### on_proposal_callback

### register_delegation
1. Check that delegation appears in 'delegations' LookupMap.
2. Can only be called by the staking_id
3. Attached deposit is handled correctly
### delegate
1. Check that amount is added correctly
2. Check that a user can't delegate more than it has
3. Check that it can only be called by the staking_id
4. Can't be called without previos registration
### undelegate
1. Check that amount is subtracted correctly
2. Check that a user can't remove more than it delegated
3. Check that it can only be called by the staking_id
4. Can't be called without previos registration

### bounty_claim
1. Check that bounty is claimed only if all conditions are met (correct deposit, deadline, claimed for the first time)
2. Adds to bounty_claimers
3. Locks the deposit
### bounty_done
1. Check that the bounty can only be called by the creator unless it expired
2. If not expired, should add correct proposal, add to bounty_claimers, claim is marked as completed
### bounty_giveup
1. Can giveup the bounty only during the forgiveness period
2. bounty_bond is returned

### internal_payout
### internal_callback_proposal_success
### internal_callback_proposal_fail
### internal_add_bounty
### internal_execute_bounty_payout

## enum RoleKind

### add_member_to_group
### remove_member_from_group

## Policy

### add_member_to_role
### remove_member_from_role

## enum VersionedPolicy

### to_policy_mut

## Proposal 

### update_votes

# Factory