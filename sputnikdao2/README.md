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
