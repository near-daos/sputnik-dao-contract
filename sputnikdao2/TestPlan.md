# SputnikDao v2

## bounties

### bounty_claim
Claims given bounty by caller with given expected duration to execute.
- The method chould panic if the bounty with given id doesn't exist
- Should panic if `attached_deposit` is not equal to the corresponding `bounty_bond`
- Should panic in case of wrong deadline
- Should panic if all bounties are claimed
- Should increase number of claims
- Should add this claim to the list of claims, done by this account
### bounty_done
Reports that bounty is done. Creates a proposal to vote for paying out the bounty.
- Should panic if the caller is not in the list of claimers
- Should panic if the list of claims for the caller of the method doesn't contain the claim with given ID
- Should panic if the bounty claim is completed
- If claim is not expired, the `bounty_done` can only be called by the claimer
- If not expired, proposal should be added, claim is marked as completed
### bounty_giveup
Gives up working on the bounty.
- Should panic if the caller is not in the list of claimers
- Should panic if the list of claims for the caller of the method doesn't contain the claim with given ID
- If within forgiveness period, `bounty_bond` should be returned
- If within forgiveness period, claim should be removed from the list of claims, done by this account

## delegation

### register_delegation
Inserts a caller to the `delegations` LookupMap with zero balance.
- Check that delegation appears in `delegations` LookupMap.
- Can only be called by the `staking_id`
- Attached deposit is handled correctly
### delegate
Adds given amount to given account as delegated weight.
- Should panic if `staking_id` is `None`
- Check that amount is added correctly
- Check that a user can't delegate more than it has
- Check that it can only be called by the `staking_id`
- Can't be called without previos registration
### undelegate
Removes given amount from given account's delegations.
- Should panic if `staking_id` is `None`
- Check that it can only be called by the `staking_id`
- Check that amount is subtracted correctly
- Check that a user can't remove more than it delegated
- Can't be called without previous registration

## lib

### migrate
Should only be called by this contract on migration. Can be used if you haven't changed contract state.
- Can only be called by the contract
- Should migrate initial state
- Should panic if contract is not initialized
### remove_blob
Remove blob from contract storage and pay back to original storer.
- Should panic if `hash` is wrong
- Can only be called by the original storer
- Blob shold be removed
- The payback should be computed correctly
### on_proposal_callback
Receiving callback after the proposal has been finalized.
- If successful, should returns bond money to the proposal originator
- If the proposal execution failed (funds didn't transfer or function call failure), should move proposal to the "Failed" state
- Works only with one callback
### store_blob
Stores attached data into blob store and returns hash of it.
- Should panic if contract is not initialized
- Should panic if the blob already exists
- Should panic if the amount of the attached deposit is not enough
- Should save the blob to the LookupMap
- Should return hash of stored data

## proposals

### add_proposal
Adds proposal to this DAO.
- Check that the method fails in case of insufficient deposit 
- Check that different kinds of `proposal` are validated correctly
- Check that only those with a permission can add the proposal
- Check that the proposal is added to the list of proposals
### act_proposal
Act on given proposal by id, if permissions allow.
- Check that only those with a permission can act on the the proposal
- Check that the method works correctly on any possible `action`
- If proposal expired during the failed state it should be marked as expired
- If the number of votes in the group has changed (new members has been added) the proposal can lose it's approved state. In this case new proposal needs to be made, this one should expire

## views
### version
Returns the version of this contract.
### get_config
Returns the config of this contract.
### get_policy
Returns policy of this contract.
### get_staking_contract
Returns the staking contract if available. Otherwise returns `None`.
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
Returns number of the bounties.
### get_bounties
Returns the bounties.
### get_bounty_claims
Returns bounty claims for given user.
### get_bounty_number_of_claims
Returns the number of claims per given bounty.
