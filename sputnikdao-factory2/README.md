# Sputnik Factory

# Deployment & Usage

## TestNet

```
near dev-deploy --wasmFile=res/sputnikdao_factory2.wasm

# bash
CONTRACT_ID="dev-1608694678554-8567049"
# fish
set CONTRACT_ID "dev-1608694678554-8567049"

# Initialize the factory.
near call $CONTRACT_ID new '{}' --accountId $CONTRACT_ID

# bash
ARGS=`echo '{"purpose": "test", "council": ["testmewell.testnet", "illia"], "bond": "1000000000000000000000000", "vote_period": "1800000000000", "grace_period": "1800000000000"}' | base64`
# fish
set ARGS (echo '{"purpose": "test", "council": ["testmewell.testnet", "illia"], "bond": "1000000000000000000000000", "vote_period": "1800000000000", "grace_period": "1800000000000"}' | base64)

# Create a new DAO with the given parameters.
near call $CONTRACT_ID create "{\"name\": \"test\", \"public_key\": null, \"args\": \"$ARGS\"}"  --accountId $CONTRACT_ID --amount 30 --gas 100000000000000

# Create a new DAO with the given parameters while having Full Access Key to the account (trusted, but useful in case of testing or upgrades)
near call $CONTRACT_ID create "{\"name\": \"test\", \"public_key\": \"<base58 of public key>\", \"args\": \"$ARGS\"}"  --accountId $CONTRACT_ID --amount 30 --gas 100000000000000

# List all created DAOs.
near view $CONTRACT_ID get_dao_list
```

## Upgrade Factory

```bash
cargo near deploy build-reproducible-wasm $CONTRACT_ID without-init-call
```

## Deploy a new SputnikDAO Contract

As of Feb 2026: The 3.2 release will enable Global Contracts support, which will significantly reduce the cost of creation of a new DAO from 5 NEAR down to just 0.025 NEAR.

```bash
(cd ../sputnikdao2 && cargo near build reproducible-wasm)

# NOTE: Global Contract deployment costs 10x more than the regular contract, so we have to attach 49 NEAR
near contract call-function as-transaction "$CONTRACT_ID" 'store' file-args ../target/near/sputnikdao2/sputnikdao2.wasm prepaid-gas '100.0 Tgas' attached-deposit '49 NEAR' sign-as "$CONTRACT_ID"

# Take a note of the returned base58-encoded Contract Code Hash and update the following command accordingly.
# Also, update the commit_id with the commit hash you are building the contract from.

near contract call-function as-transaction "$CONTRACT_ID" 'store_contract_metadata' json-args '{"code_hash": "PUT_CONTRACT_CODE_HASH_FROM_ABOVE_COMMAND", "metadata": {"version": [3, 2], "commit_id": "PUT_THE_COMMIT_HASH_HERE"}, "set_default": true}' prepaid-gas '100.0 Tgas' attached-deposit '0 NEAR' sign-as "$CONTRACT_ID"
```

## Upgrade Old DAO to use Global Contract

Switching to the latest contract adds a few useful fields when viewing the proposals, but also it reduces the storage locked amount, so DAOs will be able to use their 5 NEAR that were deposited there long time ago.

Since it is not a security hotfix, the factory should not enforce the upgrade, so here is how each DAO can vote to upgrade itself:

```bash
# Get the default contract code hash after the factory upgrade is completed:
near contract call-function as-read-only 'sputnik-dao.near' get_default_code_hash json-args '{}' network-config mainnet now

# Create two proposals at once (sign the transactions on behalf of the DAO member when prompted):
near contract call-function as-transaction 'my-dao.sputnik-dao.near' add_proposal json-args '{"proposal": {"description": "Upgrade Sputnik-DAO Contract to 3.2 release", "kind": {"UpgradeSelf": {"hash": "PUT_CONTRACT_CODE_HASH_FROM_ABOVE_COMMAND"}}}}' prepaid-gas '300.0 Tgas' attached-deposit '0 NEAR'
near contract call-function as-transaction 'my-dao.sputnik-dao.near' add_proposal json-args '{"proposal": {"description": "Upgrade Sputnik-DAO Contract to 3.2 release once again to redeploy it as Global Contract", "kind": {"UpgradeSelf": {"hash": "PUT_CONTRACT_CODE_HASH_FROM_ABOVE_COMMAND"}}}}' prepaid-gas '300.0 Tgas' attached-deposit '0 NEAR'

# Approve those proposals either via some DAO UI (e.g. https://trezu.app) or from CLI:
near contract call-function as-transaction 'my-dao.sputnik-dao.near' act_proposal json-args '{"id": 101, "action": "VoteApprove", "proposal": {"UpgradeSelf": {"hash": "PUT_CONTRACT_CODE_HASH_FROM_ABOVE_COMMAND"}}}' prepaid-gas '100.0 Tgas' attached-deposit '0 NEAR'
near contract call-function as-transaction 'my-dao.sputnik-dao.near' act_proposal json-args '{"id": 102, "action": "VoteApprove", "proposal": {"UpgradeSelf": {"hash": "PUT_CONTRACT_CODE_HASH_FROM_ABOVE_COMMAND"}}}' prepaid-gas '100.0 Tgas' attached-deposit '0 NEAR'
```

# ABIs

V1 is archived in a different repo. :)

### Sputnik Factory :: v3

```
{
  "viewMethods": [
    "get_dao_list",
    "get_number_daos",
    "get_daos",
    "get_owner",
    "get_default_code_hash",
    "get_default_version",
    "get_code",
    "get_contracts_metadata"
  ],
  "changeMethods": [
    "new",
    "create",
    "set_owner",
    "set_default_code_hash",
    "delete_contract",
    "update",
    "store_contract_metadata",
    "delete_contract_metadata",
    "store"
  ],
}
```

### Sputnik DAO :: v2

```
{
  "viewMethods": [
    "get_dao_list"
  ],
  "changeMethods": [
    "new",
    "create"
  ],
}
```
