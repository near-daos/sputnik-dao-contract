#!/bin/bash
#### --------------------------------------------
#### NOTE: The following flows are supported in this file, for testing!
# - Create an Upgradeable DAO via sputnikv2.testnet, for testing v2-v3 upgrade
# - Upgradeable DAO store_blob
# - Upgradeable DAO proposal UpgradeSelf with hash from UpgradeDAO store_blob
# - Check code_hash on Upgradeable DAO
#### --------------------------------------------
set -e

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
export FACTORY_ACCOUNT_ID=factory19.$NEAR_ACCT
export CACHE_ACCOUNT_ID=cache4.$NEAR_ACCT
# export DAO_ACCOUNT_ID=croncat.sputnikv2.$FACTORY
# export DAO_ACCOUNT_ID=sputnikdao-dev-v2-1645228499.factory3.sputnikpm.testnet
export MAX_GAS=300000000000000
export GAS_100_TGAS=100000000000000
export GAS_150_TGAS=150000000000000
export GAS_220_TGAS=220000000000000
BOND_AMOUNT=1
BYTE_STORAGE_COST=6000000000000000000000000
V3_CODE_HASH=BKB9dViFK5uAisZwE6EooqzNX3tDBermdE1fdgRMVht2


# #### --------------------------------------------
# #### New Factory for entire test
# #### --------------------------------------------
near create-account $FACTORY_ACCOUNT_ID --masterAccount $NEAR_ACCT --initialBalance 80
near create-account $CACHE_ACCOUNT_ID --masterAccount $NEAR_ACCT --initialBalance 80
# #### --------------------------------------------



#### --------------------------------------------
#### Deploy the Upgrade Cache Contract
#### --------------------------------------------
near deploy --wasmFile sputnikdao-upgrade-cache/res/sputnikdao_upgrade_cache.wasm --accountId $CACHE_ACCOUNT_ID --initFunction new --initArgs '{}'
near view $CACHE_ACCOUNT_ID get_code_hash
#### --------------------------------------------



#### --------------------------------------------
#### Grab the factory v2 code data
#### --------------------------------------------
http --json post https://rpc.testnet.near.org jsonrpc=2.0 id=dontcare method=query \
params:='{"request_type":"view_code","finality":"final","account_id":"'sputnikv2.$FACTORY'"}' \
| jq -r .result.code_base64 \
| base64 --decode > sputnikdao_factory2_original.wasm

# Deploy the previous version to allow accurate testing
near deploy --wasmFile sputnikdao_factory2_original.wasm --accountId $FACTORY_ACCOUNT_ID --initFunction new --initArgs '{}' --initGas $MAX_GAS
#### --------------------------------------------



#### --------------------------------------------
#### Deploy Upgradeable DAO
#### --------------------------------------------
COUNCIL='["'$NEAR_ACCT'"]'
TIMESTAMP=$(date +"%s")
DAO_NAME=upgrademe-1-$TIMESTAMP
DAO_ARGS=`echo '{"config": {"name": "'$DAO_NAME'", "purpose": "A v2 dao that gets upgraded by self from remote submitted bytes", "metadata":""}, "policy": '$COUNCIL'}' | base64`
near call $FACTORY_ACCOUNT_ID create "{\"name\": \"$DAO_NAME\", \"args\": \"$DAO_ARGS\"}" --accountId $FACTORY_ACCOUNT_ID --gas $GAS_150_TGAS --amount 12
UPGRDADEME_ACCOUNT=$DAO_NAME.$FACTORY_ACCOUNT_ID
#### --------------------------------------------


#### --------------------------------------------
#### Upgradeable DAO Proposal
#### --------------------------------------------
# propose UpgradeSelf using the code_hash from store_blob
near call $UPGRDADEME_ACCOUNT add_proposal '{
  "proposal": {
    "description": "Upgrade to v3 DAO code using upgrade cache contract",
    "kind": {
      "FunctionCall": {
        "receiver_id": "'$CACHE_ACCOUNT_ID'",
        "actions": [
          {
            "method_name": "store_contract_self",
            "args": "'$FIXED_SUB_ARGS'",
            "deposit": "'$BYTE_STORAGE_COST'",
            "gas": "'$GAS_220_TGAS'"
          }
        ]
      }
    }
  }
}' --accountId $NEAR_ACCT --amount $BOND_AMOUNT --gas $MAX_GAS
# approve
near call $UPGRDADEME_ACCOUNT act_proposal '{"id": 0, "action" :"VoteApprove"}' --accountId $NEAR_ACCT  --gas $MAX_GAS
# quick check all is good
near view $UPGRDADEME_ACCOUNT get_proposal '{"id": 0}'
#### --------------------------------------------


#### --------------------------------------------
#### Upgradeable DAO Proposal
#### --------------------------------------------
# propose UpgradeSelf using the code_hash from store_blob
near call $UPGRDADEME_ACCOUNT add_proposal '{
  "proposal": {
    "description": "Upgrade to v3 DAO code using stored code",
    "kind": {
      "UpgradeSelf": {
        "hash": "'$V3_CODE_HASH'"
      }
    }
  }
}' --accountId $NEAR_ACCT --amount $BOND_AMOUNT --gas $MAX_GAS
# approve
near call $UPGRDADEME_ACCOUNT act_proposal '{"id": 1, "action" :"VoteApprove"}' --accountId $NEAR_ACCT  --gas $MAX_GAS
# quick check all is good
near view $UPGRDADEME_ACCOUNT get_proposal '{"id": 1}'
#### --------------------------------------------

# # #### --------------------------------------------
# # cleanup local files!
# # #### --------------------------------------------
# rm sputnikdao_factory2_original.wasm
# rm v3_code_hash_result.txt

echo "Dev: Go to https://explorer.testnet.near.org/accounts/$UPGRDADEME_ACCOUNT and check the code_hash matches $CODE_HASH"