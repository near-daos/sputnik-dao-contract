# Sputnik Test Planning

The following is coverage checklists & notes about context tests needed to check the security of the Sputnik DAO contracts. These tests are aimed at 100% of the latest version, and best coverage of older versions.

TC: TODO:
- add context cases
- add factory context cases

# Sputnik Factory

## Init & Default
### new
- [ ] TODO: 

## DAOs
### Creation
- [ ] TODO: 
### Upgrades
- [ ] TODO: 

## Ownership
### Changing Owner
- [ ] TODO: 
### Adding Code Version
- [ ] TODO: 
### Adding Code Metadata
- [ ] TODO: 
### Removing Code Version
- [ ] TODO: 

## views
### get_dao_list
- [ ] TODO: 
### get_number_daos
- [ ] TODO: 
### get_daos
- [ ] TODO: 
### get_owner
- [ ] TODO: 
### get_default_code_hash
- [ ] TODO: 
### get_default_version
- [ ] TODO: 
### get_code
- [ ] TODO: 
### get_contracts_metadata
- [ ] TODO: 

# Sputnik DAO

## Dao Policy Configurations

### Default
- [ ] TODO: 

### Threshold
- [ ] TODO: 

### Role Weighted
- [ ] TODO: 

### Groups Weighted
- [ ] TODO: 

### Token Weighted
- [ ] TODO: 

### Groups Varying Policy
- [ ] TODO: 

## Staking Token

### None
- [ ] TODO: 

### New Staking Contract
- [ ] TODO: 


## bounties

### Happy path
Creates an end-to-end check of happy path completion
- [x] Setup test token
- [x] propose a bounty
- [x] Vote on the bounty
- [x] Claim the bounty
- [x] Check bounty has claims
- [x] Make bounty done
- [x] Check bounty has claims
- [x] Finalize bounty proposal
- [x] Check bounty proposal approved

### bounty_claim
Claims given bounty by caller with given expected duration to execute.
- [x] The method could panic if the bounty with given id doesn't exist
- [x] Should panic if `attached_deposit` is not equal to the corresponding `bounty_bond`
- [x] Should panic in case of wrong deadline
- [x] Should panic if all bounties are claimed
- [x] Should increase number of claims
- [x] Should add this claim to the list of claims, done by this account
### bounty_done
Reports that bounty is done. Creates a proposal to vote for paying out the bounty.
- [x] Should panic if the caller is not in the list of claimers
- [x] Should panic if the list of claims for the caller of the method doesn't contain the claim with given ID
- [x] Should panic if the bounty claim is completed
- [x] If claim is not expired, the `bounty_done` can only be called by the claimer
- [x] If not expired, proposal should be added, claim is marked as completed
### bounty_giveup
Gives up working on the bounty.
- [x] Should panic if the caller is not in the list of claimers
- [x] Should panic if the list of claims for the caller of the method doesn't contain the claim with given ID
- [x] If within forgiveness period, `bounty_bond` should be returned
- [x] If within forgiveness period, claim should be removed from the list of claims, done by this account

## delegation

### register_delegation
Inserts a caller to the `delegations` LookupMap with zero balance.
- [x] Check that delegation appears in `delegations` LookupMap.
- [x] Can only be called by the `staking_id`
- [x] Attached deposit is handled correctly
### delegate
Adds given amount to given account as delegated weight.
- [x] Should panic if `staking_id` is `None`
- [x] Check that amount is added correctly
- [x] Check that a user can't delegate more than it has
- [x] Check that it can only be called by the `staking_id`
- [x] Can't be called without previos registration
### undelegate
Removes given amount from given account's delegations.
- [x] Should panic if `staking_id` is `None`
- [x] Check that it can only be called by the `staking_id`
- [x] Check that amount is subtracted correctly
- [x] Check that a user can't remove more than it delegated
- [x] Can't be called without previous registration

## lib

_NOTE: This covers v2 functionality for upgrades only_
_TODO: Missing migrate, new_

### store_blob
Stores attached data into blob store and returns the hash of it.
- [x] Should panic if contract is not initialized
- [x] Should panic if the blob already exists
- [x] Should panic if the amount of the attached deposit is not enough
- [x] Should save the blob to the LookupMap
### remove_blob
Removes blob from contract storage and pays back to the original storer.
- [x] Should panic if `hash` is wrong
- [x] Should return hash of stored data
- [x] Can only be called by the original storer
- [x] Blob should be removed
- [x] The payback should be computed correctly

## Policy

_TODO: Policy is missing a lot of coverage:_

### TokenWeight
Happy path for token-weighted policy
- [x] Can create new DAO
- [x] Can set staking contract
- [x] Can change policy to TokenWeight
- [x] Can register & delegate tokens
- [x] Can use TokenWeight policy to vote & approve a proposal
### TokenWeight Self-Lock
- [x] Can create new DAO, with TokenWeight set without staking contract id
- [x] Attempt a proposal, fail to move status because voting is locked

## proposals

### add_proposal
Adds proposal to this DAO.
- [x] Check that the method fails in case of insufficient deposit 
- [x] Check that different kinds of `proposal` are validated correctly
- [x] Check that only those with a permission can add the proposal
- [x] Check that the proposal is added to the list of proposals
### act_proposal
Act on given proposal by id, if permissions allow.
- [??] Check that only those with a permission can act on the the proposal
- [x] Check that the method works correctly on any possible `action`
- [x] If the number of votes in the group has changed (new members has been added) the proposal can lose it's approved state. In this case new proposal needs to be made, this one should expire
### on_proposal_callback
Receiving callback after the proposal has been finalized.
- [??] If successful, should return bond money to the proposal originator
- [??] If the proposal execution failed (funds didn't transfer or function call failure), should move the proposal to the "Failed" state

_NOTE: Appears views are currently just helper methods and dont have test coverage_

## views
### version
- [ ] Returns the version of this contract.
### get_config
- [ ] Returns the config of this contract.
### get_policy
- [ ] Returns policy of this contract.
### get_staking_contract
- [ ] Returns the staking contract if available. Otherwise returns `None`.
### has_blob
- [ ] Returns whether the blob with given hash is stored.
### get_locked_storage_amount
- [ ] Returns the locked amount of NEAR that is used for the storage.
### get_available_amount
- [ ] Returns the available amount of NEAR that can be spent (outside of the amount for the storage and bonds).
### delegation_total_supply
- [ ] Returns the total delegated stake.
### delegation_balance_of
- [ ] Returns the delegated stake of the given account.
### delegation_balance_ratio
- [ ] Combines the balance and the total amount for calling from external contracts.
### get_last_proposal_id
- [ ] Returns the last proposal's id.
### get_proposals
- [ ] Returns a vector of the proposals.
### get_proposal
- [ ] Returns the specific proposal by id.
  - [ ] Should panic if the proposal with the given id doesn't exist
### get_bounty
- [ ] Returns the specific bounty by id.
  - [ ] Should panic if the bounty with the given id doesn't exist
### get_last_bounty_id
- [ ] Returns number of the bounties.
### get_bounties
- [ ] Returns the bounties.
### get_bounty_claims
- [ ] Returns bounty claims for given user.
### get_bounty_number_of_claims
- [ ] Returns the number of claims per given bounty.
