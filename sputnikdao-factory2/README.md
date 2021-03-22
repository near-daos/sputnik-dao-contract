# SputnikDAO Factory

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
