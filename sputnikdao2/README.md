# Sputnik DAO v2

// TODO: add short intro blurb

> Sputnik DAO v2 ...

## Overview

| Name                          | Description                                               |
| ----------------------------- | --------------------------------------------------------- |
| [Setup](#setup)               | Step-by-step guide to deploy DAO factory and DAO contract |
|[Roles & Permissions](#roles-and-permissions)||
| [Proposals](#proposals)       |                                                           |
|[Voting Policy](#voting-policy)||
| [Token Voting](#token-voting) |                                                           |
| [Bounties](#bounties)         |                                                           |
| [Blob Storage](#blob-storage) |                                                           |
| [Examples](#examples)                    |                                                           |

---

## Prerequisites

1. [NEAR Account](https://wallet.testnet.near.org)
2. [NEAR-CLI](https://docs.near.org/docs/tools/near-cli#setup)
3. [Rust](https://www.rust-lang.org)

<details>
<summary>3-Step Rust Installation.</summary>
<p>

1. Install Rustup:

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

[_(Taken from official installation guide)_](https://www.rust-lang.org/tools/install)

2. Configure your current shell:

```
source $HOME/.cargo/env
```

3. Add Wasm target to your toolchain:

```
rustup target add wasm32-unknown-unknown
```

</p>
</details>

---

## Setup

<details>
<summary>1. Login with your account.</summary>
<p>

Using [`near-cli`](https://docs.near.org/docs/tools/near-cli#near-login), login to your account which will save your credentials locally:

```
near login
```

</p>
</details>

<details>
<summary>2. Clone repository.</summary>
<p>

```
git clone https://github.com/near-daos/sputnik-dao-contract
```

</p>
</details>

<details>
<summary>3. Build factory contract.</summary>
<p>

```
cd sputnik-dao-contract/sputnikdao-factory2 && ./build.sh
```

</p>
</details>

<details>
<summary>4. Deploy factory.</summary>
<p>

- Create an env variable replacing `YOUR_ACCOUNT.testnet` with the name of the account you logged in with earlier:

```
export CONTRACT_ID=YOUR_ACCOUNT.testnet
```

- Deploy factory contract by running the following command from your current directory _(`sputnik-dao-contract/sputnikdao-factory2`)_:

```
near deploy $CONTRACT_ID --wasmFile=res/sputnikdao_factory2.wasm --accountId $CONTRACT_ID
```

</p>
</details>

<details>
<summary>5. Initialize factory.</summary>
<p>

```
near call $CONTRACT_ID new --accountId $CONTRACT_ID
```

</p>
</details>

<details>
<summary>6. Define the parameters of the new DAO, its council, and create it.</summary>
<p>

- Define the council of your DAO:

```
export COUNCIL='["council-member.testnet", "YOUR_ACCOUNT.testnet"]'
```

- Configure the name, purpose, and initial council members of the DAO and convert the arguments in base64:

```
export ARGS=`echo '{"config": {"name": "genesis", "purpose": "Genesis DAO", "metadata":""}, "policy": '$COUNCIL'}' | base64`
```

- Create the new DAO!:

```
near call $CONTRACT_ID create "{\"name\": \"genesis\", \"args\": \"$ARGS\"}" --accountId $CONTRACT_ID --amount 5 --gas 150000000000000
```

**Example Response:**

```bash
Scheduling a call: sputnik-v2.testnet.create({"name": "genesis", "args": "eyJjb25maWciOiB7Im5hbWUiOiAiZ2VuZXNpcyIsICJwdXJwb3NlIjogIkdlbmVzaXMgREFPIiwgIm1ldGFkYXRhIjoiIn0sICJwb2xpY3kiOiBbImNvdW5jaWwtbWVtYmVyLnRlc3RuZXQiLCAiWU9VUl9BQ0NPVU5ULnRlc3RuZXQiXX0K"}) with attached 5 NEAR
Transaction Id 5beqy8ZMkzpzw7bTLPMv6qswukqqowfzYXZnMAitRVS7
To see the transaction in the transaction explorer, please open this url in your browser
https://explorer.testnet.near.org/transactions/5beqy8ZMkzpzw7bTLPMv6qswukqqowfzYXZnMAitRVS7
true
```

**Note:** If you see `false` at the bottom (after the transaction link) something went wrong. Check your arguments passed and target contracts and re-deploy.

</p>
</details>

<details>
<summary>7. Verify successful deployment and policy configuration.</summary>
<p>

The DAO deployment will create a new [sub-account](https://docs.near.org/docs/concepts/account#subaccounts) ( `genesis.YOUR_ACCOUNT.testnet` ) and deploy a Sputnik v2 DAO contract to it.

- Setup another env variable for your DAO contract:

```
export SPUTNIK_ID=genesis.$CONTRACT_ID
```

- Now call `get_policy` on this contract using [`near view`](https://docs.near.org/docs/tools/near-cli#near-view)

```
near view $SPUTNIK_ID get_policy
```

- Verify that the name, purpose, metadata, and council are all configured correctly. Also note the following default values:

```json
{
  "roles": [
    {
      "name": "all",
      "kind": "Everyone",
      "permissions": ["*:AddProposal"],
      "vote_policy": {}
    },
    {
      "name": "council",
      "kind": { "Group": ["council-member.testnet", "YOUR_ACCOUNT.testnet"] },
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
    "threshold": [1, 2]
  },
  "proposal_bond": "1000000000000000000000000",
  "proposal_period": "604800000000000",
  "bounty_bond": "1000000000000000000000000",
  "bounty_forgiveness_period": "86400000000000"
}
```

</p>
</details>

---


## Roles and Permissions

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

- For example, a role with: `["mint:VoteReject","mint:VoteRemove"]` means the users with that role can only vote to _reject or remove a mint proposal_, but they can't vote to approve.

- You can use `*` as a wildcard, so for example a role with `mint:*` can perform any vote action on mint proposals.

- You can also use `*:*` for unlimited permission, normally the `council` role has `*:*` as its configured permission so they can perform any vote action on any kind of proposal.

---

## Proposals

Proposals is the main way to interact with the DAO.
Each action on the DAO is done by creating and approving proposal.


### Proposal Kinds

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

---

## Voting Policy

You can set a different vote policy for each one of the proposal kinds.

Vote policy can be: `TokenWeight`, meaning members vote with tokens, or `RoleWeight(role)` where all users with such role (e.g."council") can vote.

Also a vote policy has a "threshold". The threshold could be a ratio. e.g. `threshold:[1,2]` => 1/2 or 50% of the votes approve the proposal, or the threshold could be a fixed number (weight), so you can say that you need 3 votes to approve a proposal disregarding the amount of people in the rol, and you can say that you need 1m tokens to approve a proposal disregarding total token supply.

When vote policy is `TokenWeight`, vote % is measured against total toke supply, and each member vote weight is based on tokens owned. So if threshold is 1/2 you need half the token supply to vote "yes" to pass a proposal.

When vote policy is `RoleWeight(role)`, vote % is measured against the count of people with that role, and each member has one vote. So if threshold is 1/2 you need half the members with the role to vote "yes" to pass a proposal.

---

## Token voting

DAO votes to select some token to become voting token (only can be done once, can't change later).

User flow is next:

- User's deposit the token into the DAO.
- They can then choose who to delegate these tokens. It can be to themself or to other users to increase their vote weight.
- When users vote for proposals, their vote is weighted by all the delegations to them.
- Undelegating will block delegating / withdrawing until one voting period passes.
- Undelegated tokens can be withdrawn by the user.

---

## Bounties

The lifecycle of a bounty is the next:

- Anyone with permission can add proposal `AddBounty` which contains the bounty information, including `token` to pay the reward in and `amount` to pay it out.
- This proposal gets voted in by the current voting policy
- After proposal passed, the bounty get added. Now it has an `id` in the bounty list. Which can be queries via `get_bounties`
- Anyone can claim a bounty by calling `bounty_claim(id, deadline)` up to `repeat` times which was specified in the bounty. This allows to have repeatative bounties or multiple working collaboratively. `deadline` specifies how long it will take the sender to complete the bounty.
- If claimer decides to give up, they can call `bounty_giveup(id)`, and within `forgiveness_period` their claim bond will be returned. After this period, their bond is kept in the DAO.
- When bounty is complete, call `bounty_done(id)`, which will start add a proposal `BountyDone` that when voted will pay to whoever done the bounty.

---

## Blob storage

DAO supports storing larger blobs of data and content indexing them by hash of the data.
This is done to allow upgrading the DAO itself and other contracts.

Blob lifecycle:

- Store blob in the DAO
- Create upgradability proposal
- Proposal passes or fails
- Remove blob and receive funds locked for storage back

Blob can be removed only by the original storer.

---

## Examples


#### Step 5. Create a proposal and interact with it:

Lets use a third user, called `another-account.testnet` to create a proposal. The proposal asks for `another-account.testnet` they joins the council. The proposal will be votable for only a minute (`"submission_time":"60000000000"`).

```
near call $SPUTNIK_ID add_proposal '{"proposal": {"description": "test", "submission_time":"60000000000", "kind": {"AddMemberToRole": {"member_id": "another-account.testnet", "role": "council"}}}}' --accountId another-account.testnet --amount 1
```

Vote "Approve" using the **council members**:

```
near call $SPUTNIK_ID act_proposal '{"id": 0, "action": "VoteApprove"}' --accountId sputnik2.testnet
near call $SPUTNIK_ID act_proposal '{"id": 0, "action": "VoteApprove"}' --accountId councilmember.testnet
```

View proposal:

```
near view $SPUTNIK_ID get_proposal '{"id": 0}'
```

After one minute, the user "another-account.testnet" will be added to the council

```
near view $SPUTNIK_ID get_policy
```

View first 10 proposals:

```
near view $SPUTNIK_ID get_proposals '{"from_index": 0, "limit": 10}'
```
