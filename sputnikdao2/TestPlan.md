# SputnikDao

## bounties

### bounty_claim
Claims given bounty by caller with given expected duration to execute.
- Check that bounty is claimed only if all conditions are met (correct deposit, deadline, claimed for the first time)
- Adds to bounty_claimers
- Locks the deposit
### bounty_done
Reports that bounty is done. Creates a proposal to vote for paying out the bounty.
- Check that the bounty can only be called by the creator unless it expired
- If not expired, should add correct proposal, add to bounty_claimers, claim is marked as completed
### bounty_giveup
Gives up working on the bounty.
- Can giveup the bounty only during the forgiveness period
- bounty_bond is returned

## delegation

### register_delegation
Inserts a caller to the 'delegations' LookupMap with zero balance.
- Check that delegation appears in 'delegations' LookupMap.
- Can only be called by the staking_id
- Attached deposit is handled correctly
### delegate
Adds given amount to given account as delegated weight.
- Check that amount is added correctly
- Check that a user can't delegate more than it has
- Check that it can only be called by the staking_id
- Can't be called without previos registration
### undelegate
Removes given amount from given account's delegations.
- Check that amount is subtracted correctly
- Check that a user can't remove more than it delegated
- Check that it can only be called by the staking_id
- Can't be called without previos registration

## lib

### migrate
Should only be called by this contract on migration. Can be used if you haven't changed contract state.
- Can only be called by the contract
- Should migrate initial state
### remove_blob
Remove blob from contract storage and pay back to original storer.
- Can only be called by the original storer
- Blob is removed
- The payback is computed correctly
### on_proposal_callback
Receiving callback after the proposal has been finalized.
- If successful, should returns bond money to the proposal originator
- If the proposal execution failed (funds didn't transfer or function call failure), should moves proposal to "Failed" state
- Works only with one callback

## proposals

### add_proposal
- Check that the proposal is added to the list of proposals
- Check that ProposalInput can have any ProposalKind (or is it not required?)
- Check that only those with a permission can add the proposal
- Chech that the method fails in case of insufficient deposit 
### act_proposal
- Check that only those with a permission can act on the the proposal
- Check that the method works correctly on all possible Actions. Also should act differently during expired or failed finalization

## views
### version
Returns the version of this contract.
### get_config
Returns the config of this contract.
### get_policy
Returns policy of this contract.
### get_staking_contract
Returns the staking contract if available. Otherwise returns None.
### has_blob
Returns whether the blob with given hash is stored.
### get_locked_storage_amount
Returns the locked amount of NEAR that is used for the storage.
### get_available_amount
Returns the available amount of NEAR that can be spent (outside of the amount for the storage and bonds).
### delegation_total_supply
Returns the total delegated stake.
### delegation_balance_of
Returns the delegated stake of the given account.
### delegation_balance_ratio
Combines the balance and the total amount for calling from external contracts.
### get_last_proposal_id
Returns the last proposal's id.
### get_proposals
Returns a vector of the proposals.
### get_proposal
Returns the specific proposal by id.
- Should panic if the proposal with the given id doesn't exist
### get_bounty
Returns the specific bounty by id.
- Should panic if the bounty with the given id doesn't exist
### get_last_bounty_id
Returns the the last bounty.
### get_bounties
Returns the bounties.
### get_bounty_claims
Returns bounty claims for given user.
### get_bounty_number_of_claims
Returns the number of claims per given bounty.








## Not in wasm file?



### internal_payout
### internal_callback_proposal_success
### internal_callback_proposal_fail
### internal_add_bounty
### internal_execute_bounty_payout
 
### add_member_to_group
### remove_member_from_group

### add_member_to_role
### remove_member_from_role

### to_policy_mut

### update_votes
