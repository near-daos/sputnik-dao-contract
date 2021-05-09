# Sputnik DAO v2

## Proposals

Proposals is the main way to interact with the DAO.
Each action on the DAO is done by creating and approving proposal.


## Bounties

The lifecycle of a bounty is the next:

 - Anyone with permission can add proposal `AddBounty` which contains the bounty information, including `token` to pay the reward in and `amount` to pay it out.
 - This proposal gets voted in by the current voting policy
 - After proposal passed, the bounty get added. Now it has an `id` in the bounty list. Which can be queries via `get_bounties`
 - Anyone can claim a bounty by calling `bounty_claim(id, deadline)` up to `repeat` times which was specified in the bounty. This allows to have repeatative bounties or multiple working collaboratively. `deadline` specifies how long it will take the sender to complete the bounty.
 - If claimer decides to give up, they can call `bounty_giveup(id)`, and within `forgiveness_period` their claim bond will be returned. After this period, their bond is kept in the DAO.
 - When bounty is complete, call `bounty_done(id)`, which will start add a proposal `BountyDone` that when voted will pay to whoever done the bounty.

## Blob storage

DAO supports storing larger blobs of data and content indexing them by hash of the data.
This is done to allow upgrading the DAO itself and other contracts.

Blob lifecycle:
 - Store blob in the DAO
 - Create upgradability proposal
 - Proposal passes or fails
 - Remove blob and receive funds locked for storage back

Blob can be removed only by the original storer.

## Testing

Use `export CONTRACT_ID=sputnik2.testnet`, to set the account to deploy the factory.

Step 1. Deploy factory:
```
near create-account ...
near deploy $CONTRACT_ID --wasmFile=sputnikdao_factory2/res/sputnikdao_factory2.wasm
```

Step 2. Initiatlize factory
```
near call $CONTRACT_ID new --accountId $CONTRACT_ID
```

Step 2. Create new Sputnik DAO:
```
# bash
ARGS=`echo '{"config": {"name": "genesis", "symbol": "GENESIS", "decimals": 24, "purpose": "test", "bond": "1000000000000000000000000", "metadata": ""}, "policy": ["testmewell.testnet", "sputnik2.testnet"]}' | base64`
# fish
set ARGS (echo '{"config": {"name": "genesis", "symbol": "GENESIS", "decimals": 24, "purpose": "test", "bond": "1000000000000000000000000", "metadata": ""}, "policy": ["testmewell.testnet", "sputnik2.testnet"]}' | base64)

# Create a new DAO with the given parameters.
near call $CONTRACT_ID create "{\"name\": \"genesis\", \"args\": \"$ARGS\"}"  --accountId $CONTRACT_ID --amount 5 --gas 150000000000000
```

Set `export SPUTNIK_ID=genesis.$CONTRACT_ID`.

Validate that it went through and current policy:
```
near view $SPUTNIK_ID get_policy
```

To create a proposal:
```
near call $SPUTNIK_ID add_proposal '{"proposal": {"description": "test", "kind": {"AddMemberToRole": {"member_id": "testmewell.testnet", "role": "council"}}}}' --accountId testmewell.testnet --amount 1
```

Vote "Approve" for the proposal:
```
near call $SPUTNIK_ID act_proposal '{"id": 0, "action": "VoteApprove"}' --accountId testmewell.testnet
```

View proposal:
```
near view $SPUTNIK_ID get_proposal '{"id": 0}'
```

View first 10 proposals:
```
near view $SPUTNIK_ID get_proposals '{"from_index": 0, "limit": 10}'
```

## Proposal Kinds

Each kind of proposal represents an operation the DAO can perform. Proposal kinds are:
```
ProposalKind::ChangeConfig { .. } => "config",
ProposalKind::ChangePolicy { .. } => "policy",
ProposalKind::AddMemberToRole { .. } => "add_member_to_role",
ProposalKind::RemoveMemberFromRole { .. } => "remove_member_from_role",
ProposalKind::FunctionCall { .. } => "call",
ProposalKind::UpgradeSelf { .. } => "upgrade_self",
ProposalKind::UpgradeRemote { .. } => "upgrade_remote",
ProposalKind::Transfer { .. } => "transfer",
ProposalKind::Mint { .. } => "mint",
ProposalKind::Burn { .. } => "burn",
ProposalKind::AddBounty { .. } => "add_bounty",
ProposalKind::BountyDone { .. } => "bounty_done",
ProposalKind::Vote => "vote",
```
### Voting Policy

You can set a different vote policy for each one of the proposal kinds.

Vote policy can be: `TokenWeight`, meaning members vote with tokens, or `RoleWeight(role)` where all users with such role (e.g."council") can vote.

Also a vote policy has a "threshold". The threshold could be a ratio. e.g. `threshold:[1,2]` => 1/2 or 50% of the votes approve the proposal, or the threshold could be a fixed number (weight), so you can say that you need 3 votes to approve a proposal disregarding the amount of people in the rol, and you can say that you need 1m tokens to approve a proposal disregarding total token supply.

When vote policy is `TokenWeight`, vote % is measured against total toke supply, and each member vote weight is based on tokens owned. So if threshold is 1/2 you need half the token supply to vote "yes" to pass a proposal.

When vote policy is `RoleWeight(role)`, vote % is measured against the count of people with that role, and each member has one vote. So if threshold is 1/2 you need half the members with the role to vote "yes" to pass a proposal.

## Roles & Permissions

The DAO can have several roles, and you can define permissions for each role. A permission is a combination of `proposal_kind:VotingAction` so they can become very specific.

Actions are:
```
/// Action to add proposal. Used internally.
AddProposal,
/// Action to remove given proposal. Used for immediate deletion in special cases.
RemoveProposal,
/// Vote to approve given proposal or bounty.
VoteApprove,
/// Vote to reject given proposal or bounty.
VoteReject,
/// Vote to remove given proposal or bounty (because it's spam).
VoteRemove,
/// Finalize proposal, called when it's expired to return the funds
/// (or in the future can be used for early proposal closure).
Finalize,
/// Move a proposal to the hub to shift into another DAO.
MoveToHub
```

so, for example a role with: `["mint:VoteReject","mint:VoteRemove"]` means the users with that role can only vote to *reject or remove a mint proposal*, but they can't vote to approve.

You can use `*` as a wildcard, so for example a role with `mint:*` can perform any vote action on mint proposals.

You can also use `*:*` for unlimited permission, normally the `council` role has `*:*` as its configured permission so they can perform any vote action on any kind of proposal.
