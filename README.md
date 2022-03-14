# Sputnik DAO

> Building on the functionality of [Sputnik V1](https://github.com/near-daos/sputnik-dao-contract-legacy), Sputnik DAO V2 offers even more features and enhanced configuration ability. Sputnik V1 is archived because it can no longer be extended. Its newer version, Sputnik V2, aims to be more flexible in this regard and it provides new features that can be opt-in by the users.

## Overview

| Name                                          | Description                                                           |
| --------------------------------------------- | --------------------------------------------------------------------- |
| [Setup](#setup)                               | Step-by-step guide to deploy a DAO factory and DAO contracts.         |
| [Roles & Permissions](#roles-and-permissions) | Setup roles and define permissions for each role.                     |
| [Proposals](#proposals)                       | Each action on the DAO is done by creating and approving a proposal.  |
| [Voting](#voting)                             | Configure policies, setup governance tokens, and vote on proposals.   |
| [Bounties](#bounties)                         | Add and configure bounties.                                           |
| [Blob Storage](#blob-storage)                 | Store large data blobs and content and index them by the data's hash. |
| [Upgradability](#upgradability)               | Upgrade the DAO to different contract code versions.                  |

---

## Setup

### Prerequisites

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

<details>
<summary>1. Login with your account.</summary>
<p>

Using [`near-cli`](https://docs.near.org/docs/tools/near-cli#near-login), login to your account which will save your credentials locally:

```bash
near login
```

</p>
</details>

<details>
<summary>2. Clone repository.</summary>
<p>

```bash
git clone https://github.com/near-daos/sputnik-dao-contract
```

</p>
</details>

<details>
<summary>3. Build factory contract.</summary>
<p>

```bash
cd sputnik-dao-contract/sputnikdao-factory2 && ./build.sh
```

</p>
</details>

<details>
<summary>4. Deploy factory.</summary>
<p>

- Create an env variable replacing `YOUR_ACCOUNT.testnet` with the name of the account you logged in with earlier:

```bash
export CONTRACT_ID=YOUR_ACCOUNT.testnet
```

- Deploy factory contract by running the following command from your current directory _(`sputnik-dao-contract/sputnikdao-factory2`)_:

```bash
near deploy $CONTRACT_ID --wasmFile=res/sputnikdao_factory2.wasm --accountId $CONTRACT_ID
```

</p>
</details>

<details>
<summary>5. Initialize factory.</summary>
<p>

```bash
near call $CONTRACT_ID new --accountId $CONTRACT_ID --gas 100000000000000
```

</p>
</details>

<details>
<summary>6. Define the parameters of the new DAO, its council, and create it.</summary>
<p>

- Define the council of your DAO:

```bash
export COUNCIL='["council-member.testnet", "YOUR_ACCOUNT.testnet"]'
```

- Configure the name, purpose, and initial council members of the DAO and convert the arguments in base64:

```bash
export ARGS=`echo '{"config": {"name": "genesis", "purpose": "Genesis DAO", "metadata":""}, "policy": '$COUNCIL'}' | base64`
```

- Create the new DAO!:

```bash
near call $CONTRACT_ID create "{\"name\": \"genesis\", \"args\": \"$ARGS\"}" --accountId $CONTRACT_ID --amount 10 --gas 150000000000000
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

```bash
export SPUTNIK_ID=genesis.$CONTRACT_ID
```

- Now call `get_policy` on this contract using [`near view`](https://docs.near.org/docs/tools/near-cli#near-view)

```bash
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

> The DAO can have several roles, each of which allows for permission configuring. These permissions are a combination of [`proposal_kind`](#proposal-types) and `VotingAction`. Due to this combination these permissions can be scoped to be very specific or you can use wildcards to grant greater access.

**Examples:**

- A role with: `["transfer:VoteReject","transfer:VoteRemove"]` means they can only vote to _reject_ or _remove_ a `transfer` proposal but they can't vote to approve.

- A role with: `["transfer:*"]` can perform any vote action on `transfer` proposals.

- A role with: `["*:*"]` has _unlimited_ permission. Normally, the `council` role has `*:*` as its permission so they can perform _any_ vote action on _any_ kind of proposal.

**Here is a list of actions:**

- `AddProposal` - _Adds given proposal to the DAO (this is the primary mechanism for getting things done)._
- `RemoveProposal` - _Removes given proposal (this is used for immediate deletion in special cases)._
- `VoteApprove` - _Votes to approve given proposal or bounty._
- `VoteReject` - _Votes to reject given proposal or bounty._
- `VoteRemove` - _Votes to remove given proposal or bounty (this may be because the proposal is spam or otherwise invalid)._
- `Finalize` - _Finalizes proposal which is cancelled when proposal has expired (this action also returns funds)._
- `MoveToHub` - _Moves a proposal to the hub (this is used to move a proposal into another DAO)._

---

## Proposals

> Proposals are the main way to interact with the DAO. Each action on the DAO is performed by creating and approving a proposal.

| Contents                                            |
| --------------------------------------------------- |
| [Proposal types](#proposal-types)                   |
| [Add proposal](#add-proposal)                       |
| [View proposal](#view-proposal)                     |
| [View multiple proposals](#view-multiple-proposals) |
| [Approve proposal](#approve-proposal)               |

---

### Proposal types

> Each kind of proposal represents an operation the DAO can perform. Here are the kinds of proposals:

```rs
ProposalKind::ChangeConfig { .. },
ProposalKind::ChangePolicy { .. },
ProposalKind::AddMemberToRole { .. },
ProposalKind::RemoveMemberFromRole { .. },
ProposalKind::FunctionCall { .. },
ProposalKind::UpgradeSelf { .. },
ProposalKind::UpgradeRemote { .. },
ProposalKind::Transfer { .. },
ProposalKind::SetStakingContract { .. },
ProposalKind::AddBounty { .. },
ProposalKind::BountyDone { .. },
ProposalKind::Vote,
ProposalKind::FactoryInfoUpdate { .. },
ProposalKind::ChangePolicyAddOrUpdateRole { .. },
ProposalKind::ChangePolicyRemoveRole { .. },
ProposalKind::ChangePolicyUpdateDefaultVotePolicy { .. },
ProposalKind::ChangePolicyUpdateParameters { .. },
```

- **ChangeConfig** - used to change the configuration of the DAO
- **ChangePolicy** - used to change the full policy of the DAO
- **AddMemberToRole** - used to add a member to a role in the DAO
- **RemoveMemberFromRole** - used to remove a member from a role in the DAO
- **FunctionCall** - used to a call a function on any valid account on the network including the DAO itself, any other DAO, or any other contract. This is a useful mechanism for extending the capabilities of the DAO without modifying or complicating the DAO contract code.  One can imagine a family of contracts built specifically to serve the DAO as agents, proxies, oracles and banks, for example.
- **UpgradeSelf** - used to upgrade the DAO contract itself.
- **UpgradeRemote** - used to upgrade other contracts. For DAOs that are governing other protocols, this type of proposal will allow to upgrade another contract with its newer version.
- **Transfer** - used to move assets from this DAO to another account on the network. Supports both `NEAR` and any `NEP-141` token that this DAO has.
- **SetStakingContract** - used to set the staking contract of the DAO to help users delegate their tokens.
- **AddBounty** - used to add a bounty to encourage members of the DAO community to contribute their time and attention to the needs of the DAO
- **BountyDone** - used to mark the completion of an available bounty
- **Vote** - used to create polls. Vote proposal doesn't have any action.
- **FactoryInfoUpdate** - used for changing permissions of the factory that created the DAO. By default, the factory has permission to upgrade the DAO, but this can be modified by using `FactoryInfoUpdate`.
- **ChangePolicyAddOrUpdateRole** - used to add a new role to the policy of the DAO. If the role already exists, update it.
- **ChangePolicyRemoveRole** - used to remove a role from the policy of the DAO.
- **ChangePolicyUpdateDefaultVotePolicy** - used to update the default vote policy from the policy of the DAO.
- **ChangePolicyUpdateParameters** - used to update the parameters from the policy of the DAO. Parameters include: proposal bond, proposal period, bounty bond, bounty forgiveness period.

---

### Add proposal

> Adds a proposal to the DAO contract and returns the index number of the proposal or "proposal ID". By default, anyone can add a proposal but it requires a minimum 1 Ⓝ bond (attached deposit).

- method: `add_proposal`
- params:
  - `proposal`
    - `description`
    - `kind`
- proposer account ID
- attached deposit (minimum 1 Ⓝ)

<details>
<summary>Example argument structure:</summary>
<p>

```json
{
  "proposal": {
    "description": "Add New Council",
    "kind": {
      "AddMemberToRole": {
        "member_id": "council_member_3.testnet",
        "role": "council"
      }
    }
  }
}
```

</p>
</details>

<details>
<summary>Example near-cli command:</summary>
<p>

```bash
near call genesis.sputnik-v2.testnet add_proposal \
'{"proposal": {"description": "Add New Council", "kind": {"AddMemberToRole": {"member_id": "council_member_3.testnet", "role": "council"}}}}' \
--accountId proposer.testnet \
--amount 1
```

</p>
</details>

<details>
<summary>Example response:</summary>
<p>

```bash
Transaction Id HbJdK9AnZrvjuuoys2z1PojdkyFiuWBvrDbXsAf5ndvu
To see the transaction in the transaction explorer, please open this url in your browser
https://explorer.testnet.near.org/transactions/HbJdK9AnZrvjuuoys2z1PojdkyFiuWBvrDbXsAf5ndvu
0
```

**Note:** The number under the transaction link is the proposal ID.

</p>
</details>

---

### View proposal

> Returns proposal details by passing the ID or index of a given proposal.

- method: `get_proposal`
  - params: `id`

<details>
<summary>Example near-cli command:</summary>
<p>

```bash
near view genesis.sputnik-v2.testnet get_proposal '{"id": 0}'
```

</p>
</details>

<details>
<summary>Example response:</summary>
<p>

```json
{
  "id": 0,
  "proposer": "near-example.testnet",
  "description": "Add New Council",
  "kind": {
    "AddMemberToRole": {
      "member_id": "council_member_3.testnet",
      "role": "council"
    }
  },
  "status": "InProgress",
  "vote_counts": {},
  "votes": {},
  "submission_time": "1624947631810665051"
}
```

</p>
</details>

---

### View multiple proposals

> Returns multiple proposal details by passing the index ("ID") starting point and a limit of how many records you would like returned.

- method: `get_proposals`
- params:
  - `from_index`
  - `limit`

<details>
<summary>Example near-cli command:</summary>
<p>

```bash
near view genesis.sputnik-v2.testnet get_proposals '{"from_index": 1, "limit": 2}'
```

</p>
</details>

<details>
<summary>Example response:</summary>
<p>

```js
[
  {
    id: 1,
    proposer: 'near-example.testnet',
    description: 'Add New Council',
    kind: {
      AddMemberToRole: { member_id: 'council_member_4.testnet', role: 'council' }
    },
    status: 'InProgress',
    vote_counts: {},
    votes: {},
  submission_time: '1624947785010147691'
  },
  {
    id: 2,
    proposer: 'near-example.testnet',
    description: 'Add New Council',
    kind: {
      AddMemberToRole: { member_id: 'council_member_5.testnet', role: 'council' }
    },
    status: 'InProgress',
    vote_counts: {},
    votes: {},
    submission_time: '1624947838518330827'
  }
]
```

</p>
</details>

---

### Approve proposal

> Approves proposal by ID. Only council members can approve a proposal

- method: `act_proposal`
- params:
  - `id`
  - `action`
- account ID that is a council member.

<details>
<summary>Example near-cli command:</summary>
<p>

```bash
near call genesis.sputnik-v2.testnet act_proposal '{"id": 0, "action": "VoteApprove"}' \
--accountId council_member_1.testnet
```

</p>
</details>

<details>
<summary>Example response:</summary>
<p>

```bash
Receipts: 3mkSgRaHsd46FHkf9AtTcPbNXkYkxMCzPfJFHsHk8NPm, GjJ6hmoAhxt2a7si4hVPYZiL9CWeM5fmSEzMTpC7URxV
        Log [genesis.sputnik-v2.testnet]: ["council"]
Transaction Id BZPHxNoBpyMG4seCojzeNrKpr685vWPynDMTdg1JACa7
To see the transaction in the transaction explorer, please open this url in your browser
https://explorer.testnet.near.org/transactions/BZPHxNoBpyMG4seCojzeNrKpr685vWPynDMTdg1JACa7
''
```

</p>
</details>

---

## Voting

>

### Vote on a proposal

> Only council members are allowed to vote on a proposal.

---

### Voting policy

> You can set a different vote policy for each one of the proposal kinds.

Vote policy can be: `TokenWeight`, meaning members vote with tokens, or `RoleWeight(role)` where all users with such role (e.g."council") can vote.

Also a vote policy has a "threshold". The threshold could be a ratio. e.g. `threshold:[1,2]` => 1/2 or 50% of the votes approve the proposal, or the threshold could be a fixed number (weight), so you can say that you need 3 votes to approve a proposal disregarding the amount of people in the role, and you can say that you need 1m tokens to approve a proposal disregarding total token supply.

When vote policy is `TokenWeight`, vote % is measured against total toke supply, and each member vote weight is based on tokens owned. So if threshold is 1/2 you need half the token supply to vote "yes" to pass a proposal.

When vote policy is `RoleWeight(role)`, vote % is measured against the count of people with that role, and each member has one vote. So if threshold is 1/2 you need half the members with the role to vote "yes" to pass a proposal.

---

### Token voting

> DAO votes to select some token to become voting token (only can be done once, can't change later).

User flow to vote with selected token:

- Users deposit the desired amount of the token to the separate staking contract defined by the DAO.
- They can then choose who to delegate these tokens. It can be to themselves or to other users to increase their vote weight.
- When users vote for proposals, their vote is weighted by all the delegations to them.
- Undelegating will block delegating / withdrawing until one voting period passes.
- Undelegated tokens can be withdrawn by the user.

---

## Bounties

> Add and configure bounties using `AddBounty` proposal.

The lifecycle of a bounty is the next:

- Anyone with permission can add proposal `AddBounty` which contains the bounty information including `token` to pay the reward in and `amount` to pay it out.
- This proposal gets voted in by the current voting policy.
- After proposal is passed, the bounty gets added. Now it has an `id` in the bounty list which can be queried via `get_bounties`.
- Anyone can claim a bounty by calling `bounty_claim(id, deadline)` up to `repeat` times which was specified in the bounty. This allows to have repetitive bounties or multiple working collaboratively.
- `deadline` specifies how long it will take the sender to complete the bounty.
- If claimer decides to give up, they can call `bounty_giveup(id)`, and within `forgiveness_period` their claim bond will be returned. After this period, their bond is forfeited and is kept in the DAO.
- When a bounty is complete, call `bounty_done(id)`, which will add a proposal `BountyDone` that, when voted, will pay to whoever completed the bounty.

---

## Blob storage

> DAO supports storing larger blobs of data and content indexing them by hash of the data. This is done to allow upgrading the DAO itself and other contracts.

Blob lifecycle:

- Store blob in the DAO.
- Create upgradability proposal.
- Proposal passes or fails.
- Remove blob and receive funds locked for storage back.

Blob can be removed only by the original storer.

---

## Upgradability

> Allow the DAO to be upgraded to different contract code versions. This allows the DAO to use a newer, more stable and faster version of the contract code. New versions usually include new features, bug fixes and improvements in performance. Downgrade to an older version is also possible.

There are two major ways to upgrade the DAO:
 - Self upgrade by storing blob on the DAO contract and then voting to UpgradeSelf
 - Upgrade from the factory - factory stores new contract and then, if allowed, it upgrades the DAO by calling `upgrade(code)`.

DAOs can explicitly vote to disable factory auto upgrades and can pull the upgrade themselves from the factory.
