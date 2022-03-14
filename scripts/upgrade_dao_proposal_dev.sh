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
BOND_AMOUNT=1
BYTE_STORAGE_COST=6000000000000000000000000
COMMIT_V3=596f27a649c5df3310e945a37a41a957492c0322
V3_CODE_HASH=FRc1X7yrgGEnjVEauMMuPTQJmzDdp3ZDfxjomkrLexzq


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
#### Quick sanity check on getters
#### --------------------------------------------
near view $FACTORY_ACCOUNT_ID get_dao_list
#### --------------------------------------------


#### --------------------------------------------
#### Upgrade the factory
#### NOTE: Make sure you've built on the right commit!
#### --------------------------------------------
near deploy --wasmFile sputnikdao-factory2/res/sputnikdao_factory2.wasm --accountId $FACTORY_ACCOUNT_ID --force
#### --------------------------------------------



#### --------------------------------------------
#### Grab the DAO v2 code data & store it in factory
#### --------------------------------------------
http --json post https://rpc.testnet.near.org jsonrpc=2.0 id=dontcare method=query \
params:='{"request_type":"view_code","finality":"final","account_id":"'$DAO_ACCOUNT_ID'"}' \
| jq -r .result.code_base64 \
| base64 --decode > sputnikdao2_original.wasm

# Store the code data
V2_BYTES='cat sputnikdao2_original.wasm | base64'
near call $FACTORY_ACCOUNT_ID store $(eval "$V2_BYTES") --base64 --accountId $FACTORY_ACCOUNT_ID --gas $GAS_100_TGAS --amount 10 > v2_code_hash_result.txt

# Update the factory metadata
# Get the response code hash!
V2_CODE_HASH=$(eval "tail -1 v2_code_hash_result.txt | sed 's/^.//;s/.$//'")
echo "V2 CODE HASH: $V2_CODE_HASH"
near call $FACTORY_ACCOUNT_ID store_contract_metadata '{"code_hash": "'$V2_CODE_HASH'", "metadata": {"version": [2,0], "commit_id": "c2cf1553b070d04eed8f659571440b27d398c588"}, "set_default": false}' --accountId $FACTORY_ACCOUNT_ID
#### --------------------------------------------



#### --------------------------------------------
#### Get DAO v3 code data & store it in factory
#### --------------------------------------------
# Store the code data
V3_BYTES='cat sputnikdao2/res/sputnikdao2.wasm | base64'
near call $FACTORY_ACCOUNT_ID store $(eval "$V3_BYTES") --base64 --accountId $FACTORY_ACCOUNT_ID --gas $GAS_100_TGAS --amount 10 > v3_code_hash_result.txt

# Update the factory metadata
# Get the response code hash!
V3_CODE_HASH=$(eval "tail -1 v3_code_hash_result.txt | sed 's/^.//;s/.$//'")
echo "V3 CODE HASH: $V3_CODE_HASH"
near call $FACTORY_ACCOUNT_ID store_contract_metadata '{"code_hash": "'$V3_CODE_HASH'", "metadata": {"version": [3,0], "commit_id": "'$COMMIT_V3'"}, "set_default": true}' --accountId $FACTORY_ACCOUNT_ID
#### --------------------------------------------



#### --------------------------------------------
#### Sanity check the new metadata
#### --------------------------------------------
near view $FACTORY_ACCOUNT_ID get_contracts_metadata
#### --------------------------------------------



#### --------------------------------------------
#### Upgradeable DAO Proposal
#### --------------------------------------------
# FRc1X7yrgGEnjVEauMMuPTQJmzDdp3ZDfxjomkrLexzq

UPGRADE_PROPOSAL_ARGS=`echo '{"code_hash":"FRc1X7yrgGEnjVEauMMuPTQJmzDdp3ZDfxjomkrLexzq"}' | base64`
# propose UpgradeSelf using the code_hash from store_blob
near call $UPGRDADEME_ACCOUNT add_proposal '{
  "proposal": {
    "description": "Upgrade to v3 DAO code using upgrade contract via factory",
    "kind": {
      "FunctionCall": {
        "receiver_id": "'$FACTORY_ACCOUNT_ID'",
        "actions": [
          {
            "method_name": "store_contract_self",
            "args": "'$UPGRADE_PROPOSAL_ARGS'",
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
V3_CODE_HASH=FRc1X7yrgGEnjVEauMMuPTQJmzDdp3ZDfxjomkrLexzq
# propose UpgradeSelf using the code_hash from store_blob
near call $UPGRDADEME_ACCOUNT add_proposal '{
  "proposal": {
    "description": "Upgrade to v3 DAO code using local stored code",
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



#### --------------------------------------------
#### Remove cached blob DAO Proposal
#### --------------------------------------------
# FRc1X7yrgGEnjVEauMMuPTQJmzDdp3ZDfxjomkrLexzq

REMOVE_PROPOSAL_ARGS=`echo '{"code_hash":"FRc1X7yrgGEnjVEauMMuPTQJmzDdp3ZDfxjomkrLexzq"}' | base64`
# propose UpgradeSelf using the code_hash from store_blob
near call $UPGRDADEME_ACCOUNT add_proposal '{
  "proposal": {
    "description": "Remove DAO upgrade contract local code blob via factory",
    "kind": {
      "FunctionCall": {
        "receiver_id": "'$FACTORY_ACCOUNT_ID'",
        "actions": [
          {
            "method_name": "remove_contract_self",
            "args": "'$REMOVE_PROPOSAL_ARGS'",
            "deposit": "0",
            "gas": "'$GAS_220_TGAS'"
          }
        ]
      }
    }
  }
}' --accountId $NEAR_ACCT --amount $BOND_AMOUNT --gas $MAX_GAS
# approve
near call $UPGRDADEME_ACCOUNT act_proposal '{"id": 2, "action" :"VoteApprove"}' --accountId $NEAR_ACCT  --gas $MAX_GAS
# quick check all is good
near view $UPGRDADEME_ACCOUNT get_proposal '{"id": 2}'
#### --------------------------------------------

# #### --------------------------------------------
# cleanup local files!
# #### --------------------------------------------
rm sputnikdao2_original.wasm
rm sputnikdao_factory2_original.wasm
rm v2_code_hash_result.txt
rm v3_code_hash_result.txt

echo "Dev: Go to https://explorer.testnet.near.org/accounts/$UPGRDADEME_ACCOUNT and check the code_hash matches $CODE_HASH"