#!/bin/bash
#### --------------------------------------------
#### NOTE: The following flows are supported in this file, for testing!
# - Create an UpgradeDAO via sputnikv2.testnet, funded with enough for 10 upgrades
# - Create an Upgradeable DAO via sputnikv2.testnet, for testing v2-v3 upgrade
# - UpgradeDAO proposal to store_blob on Upgradeable DAO
# - Upgradeable DAO proposal UpgradeSelf with hash from UpgradeDAO store_blob
# - Check code_hash on Upgradeable DAO
#### --------------------------------------------
set -e

# # TODO: Change to the official approved commit:
# COMMIT_V3=596f27a649c5df3310e945a37a41a957492c0322
# # git checkout $COMMIT_V3

# build the things
./build.sh

export NEAR_ENV=testnet
export FACTORY=testnet

if [ -z ${NEAR_ACCT+x} ]; then
  # export NEAR_ACCT=sputnikv2.$FACTORY
  export NEAR_ACCT=sputnikpm.$FACTORY
else
  export NEAR_ACCT=$NEAR_ACCT
fi

# export FACTORY_ACCOUNT_ID=sputnikv2.$FACTORY
export FACTORY_ACCOUNT_ID=factory_1.$NEAR_ACCT
# export DAO_ACCOUNT_ID=croncat.sputnikv2.$FACTORY
export MAX_GAS=300000000000000
export GAS_100_TGAS=100000000000000
export GAS_150_TGAS=150000000000000
export GAS_220_TGAS=220000000000000
export BOND_AMOUNT=1
export BYTE_STORAGE_COST=10000000000000000000000000


# #### --------------------------------------------
# #### New Factory for entire test
# #### --------------------------------------------
near create-account $FACTORY_ACCOUNT_ID --masterAccount $NEAR_ACCT --initialBalance 80
# #### --------------------------------------------

# #### --------------------------------------------
# #### Build and deploy factory
# - Build and Deploy factory contract by running the following command from your current directory _(`sputnik-dao-contract/sputnikdao-factory2`)_:
# - Initialize factory
# #### --------------------------------------------
near deploy $FACTORY_ACCOUNT_ID --wasmFile=res/sputnikdao_factory2.wasm --accountId $FACTORY_ACCOUNT_ID
near call $FACTORY_ACCOUNT_ID new --accountId $FACTORY_ACCOUNT_ID --gas 100000000000000
# #### --------------------------------------------


#### --------------------------------------------
#### Deploy DAO that can upgrade factory
#### --------------------------------------------
export COUNCIL='["'$NEAR_ACCT'"]'
export TIMESTAMP=$(date +"%s")
export DAO_NAME=upgrademe-1-$TIMESTAMP
export DAO_ARGS=`echo '{"config": {"name": "'$DAO_NAME'", "purpose": "A dao that can upgrade a factory", "metadata":""}, "policy": '$COUNCIL'}' | base64`
near call $FACTORY_ACCOUNT_ID create "{\"name\": \"$DAO_NAME\", \"args\": \"$DAO_ARGS\"}" --accountId $FACTORY_ACCOUNT_ID --gas $GAS_150_TGAS --amount 12
export GENDAO=$DAO_NAME.$FACTORY_ACCOUNT_ID
#### --------------------------------------------


#### --------------------------------------------
#### Quick sanity check on getters
#### --------------------------------------------
near view $FACTORY_ACCOUNT_ID get_dao_list
#### --------------------------------------------

#### --------------------------------------------
#### Set owner of factory as DAO
#### --------------------------------------------
near call $FACTORY_ACCOUNT_ID set_owner '{"owner_id":"'$GENDAO'"}' --accountId $FACTORY_ACCOUNT_ID
#### --------------------------------------------

#### --------------------------------------------
#### Modify factory, rebuild, and create proposal to store
#### --------------------------------------------
# Store the code data
export FACTORYCODE='cat sputnikdao_factory2.wasm | base64'
# - most likely need to use near-cli-rs due to wasm string size limit
# Store blob in DAO
echo '{ "proposal": { "description": "Store upgrade", "kind": { "FunctionCall": { "receiver_id": "'$GENDAO'", "actions": [ { "method_name": "store_blob", "args": "'$(eval $FACTORYCODE)'", "deposit": "'$BYTE_STORAGE_COST'", "gas": "'$GAS_150_TGAS'" } ]}}}}' | base64 | pbcopy
# near call $GENDAO store $(eval "$FACTORYCODE") --base64 --accountId $FACTORY_ACCOUNT_ID --gas $GAS_100_TGAS --amount 10 > new_factory_hash.txt
# Once proposal created on Genesis DAO that is owner of factory account now, vote on it so that it can act_proposal storing the new factory code and returning a hash
# Vote on proposal
near call $GENDAO act_proposal '{"id": 0, "action" :"VoteApprove"}' --accountId $CONTRACT_ID --gas $MAX_GAS
# Act proposal might fail due to exceeded pre paid gas limit but factory is stored
# Set factory hash
export FACTORY_HASH=""

#### --------------------------------------------
#### Create proposal to upgrade factory to new factory hash
#### --------------------------------------------
near call $GENDAO add_proposal '{
  "proposal": {
    "description": "Upgrade to new factory hash using local stored code",
    "kind": {
      "UpgradeRemote": {
        "receiver_id": "spudnike.testnet",
        "method_name": "upgrade_factory",
        "hash": "'$FACTORY_HASH'"
      }
    }
  }
}' --accountId $CONTRACT_ID --amount $BOND_AMOUNT --gas $MAX_GAS
# Vote on proposal
near call $GENDAO act_proposal '{"id": 2, "action" :"VoteApprove"}' --accountId $CONTRACT_ID --gas $MAX_GAS

# Factory should be pointing to new hash

# Optionally store factory metadata
export PROPOSALARGS=`echo '{"factory_hash": "'$FACTORY_HASH'", "metadata": {"version": [1,0], "commit_id": ""}, "set_default": true}' | base64`
near call $GENDAO add_proposal '{
  "proposal": {
    "description": "Store factory metadata",
    "kind": {
      "FunctionCall": {
        "receiver_id": "'$CONTRACT_ID'",
        "actions": [
          {
            "method_name": "store_factory_metadata",
            "args": "'$PROPOSALARGS'",
            "deposit": "'$BYTE_STORAGE_COST'",
            "gas": "'$GAS_220_TGAS'"
          }
        ]
      }
    }
  }
}' --accountId $CONTRACT_ID --amount $BOND_AMOUNT --gas $MAX_GAS

# Vote on storing factory metadata

near call $GENDAO act_proposal '{"id": 1, "action" :"VoteApprove"}' --accountId $CONTRACT_ID --gas $MAX_GAS
#### --------------------------------------------
