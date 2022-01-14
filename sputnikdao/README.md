# Sputnik DAO Contracts

A simple version of a DAO to give out tips, bounties and grants.
Allows anyone to send a proposal to reward other people with funds and get a council to vote for it.

The major difference with Moloch DAO design is that this contract would receive its function via donation and council has equal rights.

Spec for v1:
 - Contract contains all the $NEAR in itself. It's initialized with it or receives later in form of donation.
 - There are council members: list of accounts that can vote for various activity. All council members have equal weight.
 - Next methods are available that can be called by anyone who attaches `bond` $NEAR (to prevent spam):
     - Add new council member
     - Remove council member
     - Given funds to `receiver` for `description` (up to 280 characters) and proposed `amount`
     - Finalize proposal
        When proposal has passed the require time, anyone can call to finalize it. Rules for passing proposal see below.
     - X of votes to approve proposal depends on the "policy": Policy allows to set number of votes required for different amount of funds spent.
 - Only council members (or self) can call:
     - `vote` for a given proposal.
 - ``Finalize proposal can be called 
        - If this vote achieves >50% of council members saying "YES" - it executes action on success.
 - Upgradability with super majority vote of the council

Voting policy is a list of amounts and number or percentage of votes required.
Where the last number in the list is used for all the non payouts (let's call it MAX_VOTE).

## Voting rules

Next rules are used for voting:
 - There is a policy that defines for `Payout` proposals at different `amount` how much "YES" votes are required. For non-`Payout` proposals - it's always 1/2 + 1 of council.
 - If there is 0 "NO" votes and given "YES" votes - the expiration date updates to current time + cooldown time.
 - If there is at least 1 "NO" then MAX_VOTE of "YES" votes is required.
 - If there is MAX_VOTE "NO" votes - the proposal gets rejected and bond not returned
 - If there is no super majority and time to withdraw have passed - proposal fails and the bond gets returned.

For example, voting policy:
  - `[(0, NumOrRation::Ration(1, 2))]` -- meaning that for any amount or vote MAX_VOTE = 1/2 + 1 is used.
  - `[(100, NumOrRation::Number(2)), (10000000, NumOrRation::Ration(2, 3))]` -- if amount is below 100 - 2 votes is enough within grace period. Otherwise MAX_VOTE = 2/3 + 1 to pass any larger proposal or add/remove council members.  

Specific examples:
  - If there are 2 councils, with default policy of 50%: proposal needs both of them to vote YES to "Succeed" or both of them to vote NO to be "Rejected". If they vote differently, the vote will be considered "Fail" and `bond` will be returned back to proposer. 

## Use cases

 - A person made a cool video about NEAR Wallet, development IDE, etc. They themself or anyone else can suggest to give them a bounty.
 - You saw really cool tweet bashing STATE bill - send that person a bounty (need them to create account though).
 - Someone contributed a small PR to one of NEAR libraries. One of maintainers can send them a bounty.
 - A person in NEAR Collective went beyond and above - another person in NEAR Collective sent them a grant.
 - Another GrantDAO applies for a grant to achieve their longer term goal via distributing to their guild members.
 - Validators have their own GrantDAO to fund ping bot or other helpful tools for validators. 

**Even better: fork this code and create a more interesting ways to distribute.**

Every guild can fork it and expand how this can be made more inclusive or more sophisticated.

## Needs

 - Nice frontend to visualize past and present proposals, creation of proposal, payouts, stats, etc.
 - This needs some form of notification service

# Development

Follow general WASM / Rust contract instructions.

## Deploy to TestNet

```bash

# Deploy to new account on TestNet
near dev-deploy res/sputnikdao.wasm

# Set contract Id (fish)
set CONTRACT_ID "dev-1608720833104-8969578"

# Initialize contract with given council and parameters (this is for testing, where you stil have access key to the contract).
# For production use either a single command or the factory in ../sputnikdao-factory 
near call $CONTRACT_ID new '{"purpose": "test", "council": ["testmewell.testnet", "illia"], "bond": "1000000000000000000000000", "vote_period": "1800000000000", "grace_period": "1800000000000"}' --accountId $CONTRACT_ID

# Get current number of proposals.
near view $CONTRACT_ID get_num_proposals

# Add new proposal to pay `illia` 1N. 
near call $CONTRACT_ID add_proposal '{"proposal": {"target": "illia", "description": "test", "kind": {"type": "Payout", "amount": "1000000000000000000000000"}}}' --accountId=illia --amount 1

# View proposal #0
near view $CONTRACT_ID get_proposal '{"id": 0}'
{
  status: 'Vote',
  proposer: 'illia',
  target: 'illia',
  description: 'test',
  kind: { Payout: { amount: '1000000000000000000000000' } },
  vote_period_end: 1607497778113967900,
  vote_yes: 0,
  vote_no: 0,
  votes: {}
}

# Get `limit=1` proposals from id=0 
near view $CONTRACT_ID get_proposals '{"from_index": 0, "limit": 1}'

# Vote for a proposal #0 `Yes` from `illia`
near call $CONTRACT_ID vote '{"id": 0, "vote": "Yes"}' --accountId illia

# Vote for a proposal #0 `No` from `testmewell.testnet`
near call $CONTRACT_ID vote '{"id": 0, "vote": "No"}' --accountId testmewell.testnet

# Proposal to add new council `testnet`.
near call $CONTRACT_ID add_proposal '{"proposal": {"target": "testnet", "description": "test", "kind": {"type": "NewCouncil"}}}' --accountId=illia --amount 1

# Proposal to remove council `illia`.
near call $CONTRACT_ID add_proposal '{"proposal": {"target": "illia", "description": "test", "kind": {"type": RemoveCouncil"}}}' --accountId=illia --amount 1

# Proposal to change vote period to 30min (in nanoseconds):
near call $CONTRACT_ID add_proposal '{"proposal": {"target": "illia", "description": "test", "kind": {"type": "ChangeVotePeriod", "vote_period": "1800000000000"}}}' --accountId=illia --amount 1

# Proposal to change purpose of this DAO:
near call $CONTRACT_ID add_proposal '{"proposal": {"target": "illia", "description": "test", "kind": {"type": "ChangePurpose", "purpose": "test me well"}}}' --accountId=illia --amount 1

# Proposal to change policy for this DAO, with next voting policy:
# - up until 100N: just need 2 votes
# - up until 1000N: need 3 votes
# - up until 2000N: need 50% + 1 votes
# - for anything larger or other types of proposals need 66% + 1 of votes
near call $CONTRACT_ID add_proposal '{"proposal": {"target": "illia", "description": "test", "kind": {"type": "ChangePolicy", "policy": [{"max_amount": "100", "votes": 2}, {"max_amount": "1000", "votes": 3}, {"max_amount": "2000", "votes": [1, 2]}, {"max_amount": "10000000", "votes": [2, 3]}]}}}' --accountId=illia --amount 1

# Finalize a proposal that has no deciding vote and expired.
near call $CONTRACT_ID finalize '{"id": 4}'
```

# Ideas for improving

## Other tokens

Add support for other tokens in the "bank".
Proposal can then specify amount in a token from whitelisted set.
There can be internal exchange function as well in case it's needed.

## Bounties

Bounties management is hard right now and done via github / notion.

Here is the idea to attach bounties to the same council:
 - Anyone can add a bounty: description + how much to pay for the bounty
 - Council votes to approve the bounty (same thing with small bounties need less votes)
 - There is a list of bounties, separate from requests
 - People can indicate that they are working on it
 - When someone completed bounty - they ping the bounty for "review" and council votes if the bounty is solved.
 - When council voted -> bounty gets paid out

## Canceling / redirecting proposals

If proposal is made to a wrong DAO, it's not great to take the bond away from proposer.
It's possible to add an option to transfer proposals from one DAO to another DAO.
Also people can vote to dismiss instead of rejecting it, which will return bond.
