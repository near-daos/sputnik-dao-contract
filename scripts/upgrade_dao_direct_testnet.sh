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
export FACTORY_ACCOUNT_ID=factory13.$NEAR_ACCT
# export DAO_ACCOUNT_ID=croncat.sputnikv2.$FACTORY
# export DAO_ACCOUNT_ID=sputnikdao-dev-v2-1645228499.factory3.sputnikpm.testnet
export MAX_GAS=300000000000000
export GAS_100_TGAS=100000000000000
export GAS_150_TGAS=150000000000000
BOND_AMOUNT=1
BYTE_STORAGE_COST=6000000000000000000000000


# #### --------------------------------------------
# #### New Factory for entire test
# #### --------------------------------------------
near create-account $FACTORY_ACCOUNT_ID --masterAccount $NEAR_ACCT --initialBalance 80
# #### --------------------------------------------



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
#### Deploy UpgradeDAO
#### --------------------------------------------
COUNCIL='["'$NEAR_ACCT'"]'
TIMESTAMP=$(date +"%s")
DAO_NAME=upgradadora-1-$TIMESTAMP
DAO_ARGS=`echo '{"config": {"name": "'$DAO_NAME'", "purpose": "A DAO to propose upgrade bytes to other DAOs", "metadata":""}, "policy": '$COUNCIL'}' | base64`
near call $FACTORY_ACCOUNT_ID create "{\"name\": \"$DAO_NAME\", \"args\": \"$DAO_ARGS\"}" --accountId $FACTORY_ACCOUNT_ID --gas $GAS_150_TGAS --amount 50
UPGRADEDAO_ACCOUNT=$DAO_NAME.$FACTORY_ACCOUNT_ID
#### --------------------------------------------



#### --------------------------------------------
#### Deploy Upgradeable DAO
#### --------------------------------------------
COUNCIL='["'$NEAR_ACCT'"]'
TIMESTAMP=$(date +"%s")
DAO_NAME=upgrademe-1-$TIMESTAMP
DAO_ARGS=`echo '{"config": {"name": "'$DAO_NAME'", "purpose": "A v2 dao that gets upgraded by self from remote submitted bytes", "metadata":""}, "policy": '$COUNCIL'}' | base64`
near call $FACTORY_ACCOUNT_ID create "{\"name\": \"$DAO_NAME\", \"args\": \"$DAO_ARGS\"}" --accountId $FACTORY_ACCOUNT_ID --gas $GAS_150_TGAS --amount 10
UPGRDADEME_ACCOUNT=$DAO_NAME.$FACTORY_ACCOUNT_ID
#### --------------------------------------------



#### --------------------------------------------
#### Upgradeable DAO Store blob
#### --------------------------------------------
# propose function call on UpgradeDAO to store_blob on Upgradeable DAO
V3_BYTES='cat sputnikdao2/res/sputnikdao2.wasm | base64'

near call $UPGRDADEME_ACCOUNT store_blob $(eval "$V3_BYTES") --base64 --accountId $NEAR_ACCT --depositYocto $BYTE_STORAGE_COST --gas $MAX_GAS > v3_code_hash_result.txt

V3_CODE_HASH=$(eval "tail -1 v3_code_hash_result.txt | sed 's/^.//;s/.$//'")
echo "V3 CODE HASH: $V3_CODE_HASH"
#### --------------------------------------------



#### --------------------------------------------
#### Upgradeable DAO Proposal
#### --------------------------------------------
# propose UpgradeSelf using the code_hash from store_blob
near call $UPGRDADEME_ACCOUNT add_proposal '{
  "proposal": {
    "description": "Upgrade to v3 DAO code using code_hash '$CODE_HASH'",
    "kind": {
      "UpgradeSelf": {
        "hash": "'$V3_CODE_HASH'"
      }
    }
  }
}' --accountId $NEAR_ACCT --amount $BOND_AMOUNT --gas $MAX_GAS
# approve
near call $UPGRDADEME_ACCOUNT act_proposal '{"id": 0, "action" :"VoteApprove"}' --accountId $NEAR_ACCT  --gas $MAX_GAS
# quick check all is good
near view $UPGRDADEME_ACCOUNT get_proposal '{"id": 0}'
#### --------------------------------------------

# #### --------------------------------------------
# cleanup local files!
# #### --------------------------------------------
rm sputnikdao_factory2_original.wasm
rm v3_code_hash_result.txt

echo "Dev: Go to https://explorer.testnet.near.org/accounts/$UPGRDADEME_ACCOUNT and check the code_hash matches $CODE_HASH"