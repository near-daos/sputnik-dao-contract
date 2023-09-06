# Sputnik Test Planning

The following is coverage checklists & notes about context tests needed to check the security of the Sputnik DAO contracts. These tests are aimed at 100% of the latest version, and best coverage of older versions.

# Sputnik Factory

## Init & Default
### new
- [ ] Can instantiate a new factory with default struct, including DAOs set.
- [ ] Stores the latest compiled version of Sputnik DAO contract in storage
- [ ] Creates metadata for the latest compiled version of Sputnik DAO
- [ ] Does not allow re-init
- [ ] Does not allow anyone but owner to call "new"

## DAOs
### Creation
- [ ] Allows any account to call create method
- [ ] Created DAO becomes a sub-account of the factory. Example for new DAO: "awesome.sputnik.near"
- [ ] Creates a new DAO & instantiates with the correct default Sputnik DAO contract from storage - see metadata
- [ ] Returns the payment amount, if the creation failed for any reason
- [ ] DAO Balance is equal to payment amount
- [ ] DAO exists in the list of DAOs upon successful creation
- [ ] Fails if the DAO name exists
- [ ] Fails if the DAO name is not a valid account ID
### Upgrades
- [ ] DAO Can update to a specific code_hash version of Sputnik DAO Code
- [ ] Fails if DAO is not within the list of supported DAOs
- [ ] Fails if DAO tries a code_hash that doesnt exist
- [ ] Fails if predecessor is not the DAO getting upgraded (DAO proposal must trigger upgrade)

## Ownership
### Changing Owner
- [ ] Can get current owner
- [ ] Fails if trying to set owner from non-owner account
- [ ] Owner can be a DAO account
- [ ] Owner gets successfully updated
### Adding Code Version
- [ ] Can store code as blob in factory
- [ ] Can set a default code_hash
- [ ] Fails if not owner of factory
- [ ] Fails if no code is attached when storing a code blob
- [ ] Fails if code blob is too small to be a legit contract
- [ ] Fails if attached payment doesnt support the storage cost
### Adding Code Metadata
- [ ] Can add metadata for an existing set of Sputnik DAO Code (code_hash is available only upon storage of contract inside factory)
- [ ] Can set the code_hash as default
- [ ] Metadata version and other params meet types & spec standards
- [ ] Fails to add code metadata if code_hash doesn't exist
- [ ] Can remove code metadata if called by owner
- [ ] Fails to remove code metadata if metadata by code_hash doesn't exist
### Removing Code Version
- [ ] Can delete a code blob by code_hash
- [ ] Can delete any/all associated code metadata for the same code_hash
- [ ] Confirm storage is empty after deletion success
- [ ] Fails if non-owner attempting to delete code blob
- [ ] Fails if no code blob exists

## views
### get_dao_list
- [ ] Returns empty array for new factory
- [ ] Returns full list of DAOs
- [ ] NOTE: This method will fail when list gets too long for gas to return on RPC
### get_number_daos
- [ ] Returns an integer representing the total amount of DAOs known to factory
### get_daos
- [ ] (Needs Impl) Returns default list of DAOs with a max length of 100 & offset of 0.
- [ ] Returns a list of DAOs matching the specified `from_index` and `limit`.
- [ ] Capable of returning non-zero indexed list, so pagination can be verified
### get_owner
- [ ] Returns a string representing the account that owns the factory
- [ ] Fails if storage is corrupted or no owner
### get_default_code_hash
- [ ] Returns the default code_hash for a new DAO
- [ ] Returns the default code_hash that has been updated after new code blob in factory
### get_default_version
- [ ] Returns the default metadata version for a new DAO, this will be a simplified semver. Example: [2,0] for V 2.0
### get_code
- [ ] Returns an entire code blob based on given code_hash
- [ ] Returns no value if code_hash doesn't exist
### get_contracts_metadata
- [ ] Returns the supported list of all factory code_hash + metadata, indicating the supported versions available for DAOs to upgrade

# Sputnik DAO

## Dao Policy Configurations
These tests are purely for checking support of certain policy configurations, no simulations.

You can check a DAO's policy by doing: 

```bash
near view DAO_NAME.sputnik-dao.near get_policy
```

### Default
**Goal:**
Confirm the default policy acts as it should.

**TESTS:**
- [ ] TODO: 

**Default Config:**

```json
{
  "roles": [
    {
      "name": "all",
      "kind": "Everyone",
      "permissions": [
        "*:AddProposal"
      ],
      "vote_policy": {}
    },
    {
      "name": "council",
      "kind": {
        "Group": [
          "user_1.testnet"
        ]
      },
      "permissions": [
        "*:Finalize",
        "*:AddProposal",
        "*:VoteApprove",
        "*:VoteReject",
        "*:VoteRemove"
      ],
      "vote_policy": {}
    }
  ],
  "default_vote_policy": {
    "weight_kind": "RoleWeight",
    "quorum": "0",
    "threshold": [ 1, 2 ]
  },
  "proposal_bond": "1000000000000000000000000",
  "proposal_period": "604800000000000",
  "bounty_bond": "1000000000000000000000000",
  "bounty_forgiveness_period": "86400000000000"
}
```

### Threshold
**Goal:**
Each 

**TESTS:**
- [ ] TODO: 

**Threshold Config:**

```json
{
  "roles": [
    {
      "name": "all",
      "kind": "Everyone",
      "permissions": [
        "*:AddProposal"
      ],
      "vote_policy": {}
    },
    {
      "name": "council",
      "kind": {
        "Group": [
          "user_1.testnet",
          "user_2.testnet",
          "user_3.testnet",
          "user_4.testnet",
          "user_5.testnet"
        ]
      },
      "permissions": [
        "*:Finalize",
        "*:AddProposal",
        "*:VoteApprove",
        "*:VoteReject",
        "*:VoteRemove"
      ],
      "vote_policy": {
        "Group":{
          "weight_kind": "RoleWeight",
          "quorum": "0",
          "threshold": [ 1, 5 ]
        }
      }
    }
  ],
  "default_vote_policy": {
    "weight_kind": "RoleWeight",
    "quorum": "0",
    "threshold": [ 1, 2 ]
  },
  "proposal_bond": "1000000000000000000000000",
  "proposal_period": "604800000000000",
  "bounty_bond": "1000000000000000000000000",
  "bounty_forgiveness_period": "86400000000000"
}
```

### Role Weighted
**Goal:**
Each 

**TESTS:**
- [ ] TODO: 

**Threshold Config:**

```json
{
  "roles": [
    {
      "name": "all",
      "kind": "Everyone",
      "permissions": [
        "*:AddProposal"
      ],
      "vote_policy": {}
    },
    {
      "name": "council",
      "kind": {
        "Group": [
          "user_1.testnet",
          "user_2.testnet",
          "user_3.testnet",
          "user_4.testnet",
          "user_5.testnet"
        ]
      },
      "permissions": [
        "*:Finalize",
        "*:AddProposal",
        "*:VoteApprove",
        "*:VoteReject",
        "*:VoteRemove"
      ],
      "vote_policy": {
        "Group":{
          "weight_kind": "RoleWeight",
          "quorum": "0",
          "threshold": [ 1, 5 ]
        }
      }
    }
  ],
  "default_vote_policy": {
    "weight_kind": "RoleWeight",
    "quorum": "0",
    "threshold": [ 1, 2 ]
  },
  "proposal_bond": "1000000000000000000000000",
  "proposal_period": "604800000000000",
  "bounty_bond": "1000000000000000000000000",
  "bounty_forgiveness_period": "86400000000000"
}
```

### Token Weighted
**Goal:**
Each 

**TESTS:**
- [ ] TODO: 

**Threshold Config:**

```json
{
  "roles": [
    {
      "name": "all",
      "kind": "Everyone",
      "permissions": [
        "*:AddProposal"
      ],
      "vote_policy": {}
    },
    {
      "name": "council",
      "kind": {
        "Group": [
          "user_1.testnet",
          "user_2.testnet",
          "user_3.testnet",
          "user_4.testnet",
          "user_5.testnet"
        ]
      },
      "permissions": [
        "*:Finalize",
        "*:AddProposal",
        "*:VoteApprove",
        "*:VoteReject",
        "*:VoteRemove"
      ],
      "vote_policy": {
        "Group":{
          "weight_kind": "RoleWeight",
          "quorum": "0",
          "threshold": [ 1, 5 ]
        }
      }
    }
  ],
  "default_vote_policy": {
    "weight_kind": "RoleWeight",
    "quorum": "0",
    "threshold": [ 1, 2 ]
  },
  "proposal_bond": "1000000000000000000000000",
  "proposal_period": "604800000000000",
  "bounty_bond": "1000000000000000000000000",
  "bounty_forgiveness_period": "86400000000000"
}
```

### Groups Weighted
**Goal:**
Each 

**TESTS:**
- [ ] TODO: 

**Threshold Config:**

```json
{
  "roles": [
    {
      "name": "all",
      "kind": "Everyone",
      "permissions": [
        "*:AddProposal"
      ],
      "vote_policy": {}
    },
    {
      "name": "council",
      "kind": {
        "Group": [
          "user_1.testnet",
          "user_2.testnet",
          "user_3.testnet",
          "user_4.testnet",
          "user_5.testnet"
        ]
      },
      "permissions": [
        "*:Finalize",
        "*:AddProposal",
        "*:VoteApprove",
        "*:VoteReject",
        "*:VoteRemove"
      ],
      "vote_policy": {
        "Group":{
          "weight_kind": "RoleWeight",
          "quorum": "0",
          "threshold": [ 1, 5 ]
        }
      }
    }
  ],
  "default_vote_policy": {
    "weight_kind": "RoleWeight",
    "quorum": "0",
    "threshold": [ 1, 2 ]
  },
  "proposal_bond": "1000000000000000000000000",
  "proposal_period": "604800000000000",
  "bounty_bond": "1000000000000000000000000",
  "bounty_forgiveness_period": "86400000000000"
}
```

### Groups Varying Policy
**Goal:**
Each group council can have different threshold criteria for consensus. Confirm that a group can be assessed based on their individual definitions versus the default policy config.

**TESTS:**
- [ ] TODO: 

**Varying Policy Config:**

```json
{
  "roles": [
    {
      "name": "council",
      "kind": {
        "Group": [
          "user_1.testnet",
          "user_2.testnet",
          "user_3.testnet",
          "user_4.testnet",
          "user_5.testnet"
        ]
      },
      "permissions": [
        "*:Finalize",
        "*:AddProposal",
        "*:VoteApprove",
        "*:VoteReject",
        "*:VoteRemove"
      ],
      "vote_policy": {
        "Group":{
          "weight_kind": "RoleWeight",
          "quorum": "0",
          "threshold": [ 1, 5 ]
        }
      }
    },
    {
      "name": "admins",
      "kind": {
        "Group": [
          "admin_1.testnet",
          "admin_2.testnet",
          "admin_3.testnet"
        ]
      },
      "permissions": [
        "*:Finalize",
        "*:AddProposal",
        "*:VoteApprove",
        "*:VoteReject",
        "*:VoteRemove"
      ],
      "vote_policy": {
        "Group":{
          "weight_kind": "RoleWeight",
          "quorum": "60",
          "threshold": []
        }
      }
    }
  ],
  "default_vote_policy": {
    "weight_kind": "RoleWeight",
    "quorum": "0",
    "threshold": [ 1, 2 ]
  },
  "proposal_bond": "1000000000000000000000000",
  "proposal_period": "604800000000000",
  "bounty_bond": "1000000000000000000000000",
  "bounty_forgiveness_period": "86400000000000"
}
```


## Staking Token
### None
- [ ] Confirming other policies means non token-staking works fine
### New Staking Contract
- [ ] Can deploy a new staking contract, configured to the right DAO owner, token & stake period
- [ ] DAO Can propose and accept the staking contract proposal
- [ ] Users can pre-pay storage & register to delegate tokens
- [ ] Users can deposit tokens using FT transfers
- [ ] Users can delegate to themselves within the staking contract
- [ ] Users can delegate to a different user within the staking contract
- [ ] Can check the amounts held within the staking contract
- [ ] Users can undelegate tokens from a delegation
- [ ] Users can withdraw any available tokens that aren't delegated in the staking contract


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
