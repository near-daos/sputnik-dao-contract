#!/bin/bash
set -e

# TODO: Change to the official approved commit:
# COMMIT_V3=596f27a649c5df3310e945a37a41a957492c0322
COMMIT_V3=TBD_SEE_COMMIT_ONCE_LIVE
# git checkout $COMMIT_V3

# build the things
./build.sh

export NEAR_ENV=testnet
export FACTORY=testnet

if [ -z ${NEAR_ACCT+x} ]; then
  export NEAR_ACCT=sputnikv2.$FACTORY
else
  export NEAR_ACCT=$NEAR_ACCT
fi

export FACTORY_ACCOUNT_ID=sputnikv2.$NEAR_ACCT
export MAX_GAS=300000000000000
export GAS_100_TGAS=100000000000000
export GAS_150_TGAS=150000000000000
BOND_AMOUNT=1
BYTE_STORAGE_COST=6000000000000000000000000
COMMIT_V2A=596f27a649c5df3310e945a37a41a957492c0322
V2A_CODE_HASH=8RMeZ5cXDap6TENxaJKtigRYf3n139iHmTRe8ZUNey6N
COMMIT_V3=596f27a649c5df3310e945a37a41a957492c0322
V3_CODE_HASH=FRc1X7yrgGEnjVEauMMuPTQJmzDdp3ZDfxjomkrLexzq


# NOTE: Only needed if deploying NEW factory
# #### --------------------------------------------
# #### Account & Data management for setup
# #### --------------------------------------------
# # near call $FACTORY_ACCOUNT_ID delete_contract '{"code_hash":"6SQymHtmezR3u9zAtLBQdb8RWCXxwxnigqSH2mRTxecB"}' --accountId $FACTORY_ACCOUNT_ID --gas $GAS_100_TGAS
# # near delete $FACTORY_ACCOUNT_ID $NEAR_ACCT
# near create-account $FACTORY_ACCOUNT_ID --masterAccount $NEAR_ACCT --initialBalance 80
# #### --------------------------------------------


# NOTE: Only needed if deploying NEW factory
# #### --------------------------------------------
# #### Grab the factory v2 code data
# #### --------------------------------------------
# http --json post https://rpc.testnet.near.org jsonrpc=2.0 id=dontcare method=query \
# params:='{"request_type":"view_code","finality":"final","account_id":"'sputnikv2.$FACTORY'"}' \
# | jq -r .result.code_base64 \
# | base64 --decode > sputnikdao_factory2_original.wasm

# # Deploy the previous version to allow accurate testing
# near deploy --wasmFile sputnikdao_factory2_original.wasm --accountId $FACTORY_ACCOUNT_ID --initFunction new --initArgs '{}' --initGas $MAX_GAS
# #### --------------------------------------------



#### --------------------------------------------
#### Deploy a v2 DAO & Some proposals
#### --------------------------------------------
COUNCIL='["'$NEAR_ACCT'"]'
TIMESTAMP=$(date +"%s")
DAO_NAME=sputnikdao-dev-v2-$TIMESTAMP
DAO_ARGS=`echo '{"config": {"name": "'$DAO_NAME'", "purpose": "Sputnik Dev v2 DAO '$TIMESTAMP'", "metadata":""}, "policy": '$COUNCIL'}' | base64`
near call $FACTORY_ACCOUNT_ID create "{\"name\": \"$DAO_NAME\", \"args\": \"$DAO_ARGS\"}" --accountId $FACTORY_ACCOUNT_ID --gas $GAS_150_TGAS --amount 10
DAO_ACCOUNT_ID=$DAO_NAME.$FACTORY_ACCOUNT_ID

# some sample payouts
near call $DAO_ACCOUNT_ID add_proposal '{"proposal": { "description": "Sample payment", "kind": { "Transfer": { "token_id": "", "receiver_id": "'$NEAR_ACCT'", "amount": "1337000000000000000000000" } } } }' --accountId $NEAR_ACCT --amount 1
near call $DAO_ACCOUNT_ID add_proposal '{"proposal": { "description": "Sample payment", "kind": { "Transfer": { "token_id": "", "receiver_id": "'$NEAR_ACCT'", "amount": "1000000000000000000000000" } } } }' --accountId $NEAR_ACCT --amount 1
near call $DAO_ACCOUNT_ID add_proposal '{"proposal": { "description": "Sample payment", "kind": { "Transfer": { "token_id": "", "receiver_id": "'$NEAR_ACCT'", "amount": "2000000000000000000000000" } } } }' --accountId $NEAR_ACCT --amount 1
# approve some, leave some
near call $DAO_ACCOUNT_ID act_proposal '{"id": 0, "action" :"VoteApprove"}' --accountId $NEAR_ACCT  --gas $MAX_GAS
near call $DAO_ACCOUNT_ID act_proposal '{"id": 1, "action" :"VoteApprove"}' --accountId $NEAR_ACCT  --gas $MAX_GAS
# quick check all is good
near view $DAO_ACCOUNT_ID get_proposal '{"id": 0}'
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
near call $FACTORY_ACCOUNT_ID store $(eval "$V2_BYTES") --base64 --accountId $FACTORY_ACCOUNT_ID --gas $MAX_GAS --amount 10 > v2_code_hash_result.txt

# Update the factory metadata
# Get the response code hash!
V2_CODE_HASH=$(eval "tail -1 v2_code_hash_result.txt | sed 's/^.//;s/.$//'")
echo "V2 CODE HASH: $V2_CODE_HASH"
near call $FACTORY_ACCOUNT_ID store_contract_metadata '{"code_hash": "'$V2_CODE_HASH'", "metadata": {"version": [2,0], "commit_id": "c2cf1553b070d04eed8f659571440b27d398c588"}, "set_default": false}' --accountId $FACTORY_ACCOUNT_ID
#### --------------------------------------------


#### --------------------------------------------
#### Get DAO v2a code data & store it in factory
#### Keep this around for gas-fixes version
#### NOTE: This doesnt really fix the upgrade path post neard 1.26.0 - those v2 DAOs will be stuck
#### --------------------------------------------
# Store the code data
V2A_BYTES='cat sputnikdao2-gasfix/res/sputnikdao2-gasfix.wasm | base64'
near call $FACTORY_ACCOUNT_ID store $(eval "$V2A_BYTES") --base64 --accountId $FACTORY_ACCOUNT_ID --gas $MAX_GAS --amount 10 > v2a_code_hash_result.txt

# Update the factory metadata
# Get the response code hash!
V2A_CODE_HASH=$(eval "tail -1 v2a_code_hash_result.txt | sed 's/^.//;s/.$//'")
echo "V2A CODE HASH: $V2A_CODE_HASH"
near call $FACTORY_ACCOUNT_ID store_contract_metadata '{"code_hash": "'$V2A_CODE_HASH'", "metadata": {"version": [2,1], "commit_id": "'$COMMIT_V2A'"}, "set_default": true}' --accountId $FACTORY_ACCOUNT_ID
#### --------------------------------------------



#### --------------------------------------------
#### Sanity check the new metadata
#### --------------------------------------------
near view $FACTORY_ACCOUNT_ID get_contracts_metadata
#### --------------------------------------------



#### --------------------------------------------
#### Get DAO v3 code data & store it in factory
#### --------------------------------------------
# Store the code data
V3_BYTES='cat sputnikdao2/res/sputnikdao2.wasm | base64'
near call $FACTORY_ACCOUNT_ID store $(eval "$V3_BYTES") --base64 --accountId $FACTORY_ACCOUNT_ID --gas $MAX_GAS --amount 10 > v3_code_hash_result.txt

# Update the factory metadata
# Get the response code hash!
V3_CODE_HASH=$(eval "tail -1 v3_code_hash_result.txt | sed 's/^.//;s/.$//'")
echo "V3 CODE HASH: $V3_CODE_HASH"
near call $FACTORY_ACCOUNT_ID store_contract_metadata '{"code_hash": "'$V3_CODE_HASH'", "metadata": {"version": [3,0], "commit_id": "'$COMMIT_V3'"}, "set_default": true}' --accountId $FACTORY_ACCOUNT_ID
#### --------------------------------------------



#### --------------------------------------------
#### Sanity check the new metadata & DAO
#### --------------------------------------------
near view $FACTORY_ACCOUNT_ID get_contracts_metadata
# Check a v2 DAO
near view $DAO_ACCOUNT_ID get_proposal '{"id": 0}'
near view $DAO_ACCOUNT_ID get_proposal '{"id": 2}'
#### --------------------------------------------



#### --------------------------------------------
#### Upgrade a v2 DAO to v2a
#### --------------------------------------------
V2A_CODE_HASH=$(eval "tail -1 v2a_code_hash_result.txt | sed 's/^.//;s/.$//'")
echo "Upgrade V2A CODE HASH: $V2A_CODE_HASH"
# some sample payouts
near call $DAO_ACCOUNT_ID add_proposal '{"proposal": { "description": "Upgrade to v2a", "kind": { "UpgradeSelf": { "hash": "'$V2A_CODE_HASH'" } } } }' --accountId $NEAR_ACCT --amount 1
# approve some, leave some
near call $DAO_ACCOUNT_ID act_proposal '{"id": 3, "action" :"VoteApprove"}' --accountId $NEAR_ACCT  --gas $MAX_GAS
# quick check all is good
near view $DAO_ACCOUNT_ID get_proposal '{"id": 0}'
near view $DAO_ACCOUNT_ID get_proposal '{"id": 3}'
#### --------------------------------------------



#### --------------------------------------------
#### Sanity check the new metadata & DAO
#### --------------------------------------------
near view $FACTORY_ACCOUNT_ID get_contracts_metadata
# Check a v2 DAO
near view $DAO_ACCOUNT_ID get_proposal '{"id": 0}'
near view $DAO_ACCOUNT_ID get_proposal '{"id": 2}'
#### --------------------------------------------



#### --------------------------------------------
#### Upgrade a v2a DAO to v3
#### --------------------------------------------
V3_CODE_HASH=$(eval "tail -1 v3_code_hash_result.txt | sed 's/^.//;s/.$//'")
echo "Upgrade V3 CODE HASH: $V3_CODE_HASH"
# some sample payouts
near call $DAO_ACCOUNT_ID add_proposal '{"proposal": { "description": "Upgrade to v3", "kind": { "UpgradeSelf": { "hash": "'$V3_CODE_HASH'" } } } }' --accountId $NEAR_ACCT --amount 1
# approve some, leave some
near call $DAO_ACCOUNT_ID act_proposal '{"id": 4, "action" :"VoteApprove"}' --accountId $NEAR_ACCT  --gas $MAX_GAS
# quick check all is good
near view $DAO_ACCOUNT_ID get_proposal '{"id": 0}'
near view $DAO_ACCOUNT_ID get_proposal '{"id": 4}'
#### --------------------------------------------



#### --------------------------------------------
#### Deploy a v3 DAO & Some proposals
#### --------------------------------------------
COUNCIL='["'$NEAR_ACCT'"]'
TIMESTAMP=$(date +"%s")
DAO_NAME=sputnikdao-dev-v3-$TIMESTAMP
DAO_ARGS=`echo '{"config": {"name": "'$DAO_NAME'", "purpose": "Sputnik Dev v3 DAO '$TIMESTAMP'", "metadata":""}, "policy": '$COUNCIL'}' | base64`
near call $FACTORY_ACCOUNT_ID create "{\"name\": \"$DAO_NAME\", \"args\": \"$DAO_ARGS\"}" --accountId $FACTORY_ACCOUNT_ID --gas $GAS_150_TGAS --amount 12
DEMO_DAO_ACCOUNT=$DAO_NAME.$FACTORY_ACCOUNT_ID

# Quick check for v3 DAO
near view $DEMO_DAO_ACCOUNT get_available_amount

# some sample payouts
near call $DEMO_DAO_ACCOUNT add_proposal '{"proposal": { "description": "Sample payment", "kind": { "Transfer": { "token_id": "", "receiver_id": "'$NEAR_ACCT'", "amount": "1000000000000000000000000" } } } }' --accountId $NEAR_ACCT --amount 1
near call $DEMO_DAO_ACCOUNT add_proposal '{"proposal": { "description": "Sample payment", "kind": { "Transfer": { "token_id": "", "receiver_id": "'$NEAR_ACCT'", "amount": "2000000000000000000000000" } } } }' --accountId $NEAR_ACCT --amount 1
near call $DEMO_DAO_ACCOUNT add_proposal '{"proposal": { "description": "Sample payment", "kind": { "Transfer": { "token_id": "", "receiver_id": "'$NEAR_ACCT'", "amount": "3000000000000000000000000" } } } }' --accountId $NEAR_ACCT --amount 1
# approve some, leave some
near call $DEMO_DAO_ACCOUNT act_proposal '{"id": 0, "action" :"VoteApprove"}' --accountId $NEAR_ACCT  --gas $MAX_GAS
near call $DEMO_DAO_ACCOUNT act_proposal '{"id": 1, "action" :"VoteApprove"}' --accountId $NEAR_ACCT  --gas $MAX_GAS
# quick check all is good
near view $DEMO_DAO_ACCOUNT get_proposal '{"id": 0}'
#### --------------------------------------------



#### --------------------------------------------
#### Sanity check v3 dao all worked
#### --------------------------------------------
near view $FACTORY_ACCOUNT_ID get_dao_list
#### --------------------------------------------

# #### --------------------------------------------
# cleanup local files!
# #### --------------------------------------------
rm sputnikdao2_original.wasm
rm sputnikdao_factory2_original.wasm
rm v2_code_hash_result.txt
rm v3_code_hash_result.txt

echo "Dev Factory Deploy & Test Complete"