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

