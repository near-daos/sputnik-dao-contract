#!/bin/bash
set -e

# TODO: Change to the official approved commit:
# COMMIT_V3=596f27a649c5df3310e945a37a41a957492c0322
COMMIT_V3=TBD_SEE_COMMIT_ONCE_LIVE
# git checkout $COMMIT_V3

# build the things
./build.sh

export NEAR_ENV=mainnet
export FACTORY=near

if [ -z ${NEAR_ACCT+x} ]; then
  export NEAR_ACCT=sputnik-dao.$FACTORY
else
  export NEAR_ACCT=$NEAR_ACCT
fi

export FACTORY_ACCOUNT_ID=sputnik-dao.$FACTORY
# export FACTORY_ACCOUNT_ID=subfactory.$NEAR_ACCT
export MAX_GAS=300000000000000
export GAS_100_TGAS=100000000000000
export GAS_150_TGAS=150000000000000
export GAS_220_TGAS=220000000000000
BOND_AMOUNT=1
BYTE_STORAGE_COST=6000000000000000000000000
COMMIT_V2A=596f27a649c5df3310e945a37a41a957492c0322
V2A_CODE_HASH=8RMeZ5cXDap6TENxaJKtigRYf3n139iHmTRe8ZUNey6N
COMMIT_V3=596f27a649c5df3310e945a37a41a957492c0322
V3_CODE_HASH=783vth3Fg8MBBGGFmRqrytQCWBpYzUcmHoCq4Mo8QqF5

DAO_ACCOUNT_ID=sputnikdao-.$FACTORY_ACCOUNT_ID

# #### --------------------------------------------
# #### Account & Data management for setup
# #### --------------------------------------------
# near call $FACTORY_ACCOUNT_ID delete_contract '{"code_hash":"'$V3_CODE_HASH'"}' --accountId $FACTORY_ACCOUNT_ID --gas $GAS_100_TGAS
# near delete $FACTORY_ACCOUNT_ID $NEAR_ACCT
# near create-account $FACTORY_ACCOUNT_ID --masterAccount $NEAR_ACCT --initialBalance 55
# #### --------------------------------------------


#### --------------------------------------------
#### Grab the factory v2 code data
#### --------------------------------------------
http --json post https://rpc.mainnet.near.org jsonrpc=2.0 id=dontcare method=query \
params:='{"request_type":"view_code","finality":"final","account_id":"'sputnik-dao.$FACTORY'"}' \
| jq -r .result.code_base64 \
| base64 --decode > sputnikdao_factory2_original.wasm

# Deploy the previous version to allow accurate testing
near deploy --wasmFile sputnikdao_factory2_original.wasm --accountId $FACTORY_ACCOUNT_ID --initFunction new --initArgs '{}' --initGas $MAX_GAS
#### --------------------------------------------


#### --------------------------------------------
#### Deploy a v2 DAO & Some proposals
#### --------------------------------------------
COUNCIL='["'$NEAR_ACCT'"]'
TIMESTAMP=$(date +"%s")
DAO_NAME=sputnikdao-$TIMESTAMP
DAO_ARGS=`echo '{"config": {"name": "'$DAO_NAME'", "purpose": "Upgrade This DAO '$TIMESTAMP'", "metadata":""}, "policy": '$COUNCIL'}' | base64`
near call $FACTORY_ACCOUNT_ID create "{\"name\": \"$DAO_NAME\", \"args\": \"$DAO_ARGS\"}" --accountId $FACTORY_ACCOUNT_ID --gas $GAS_150_TGAS --amount 6
DAO_ACCOUNT_ID=$DAO_NAME.$FACTORY_ACCOUNT_ID

# some sample payouts
near call $DAO_ACCOUNT_ID add_proposal '{"proposal": { "description": "Sample payment", "kind": { "Transfer": { "token_id": "", "receiver_id": "'$NEAR_ACCT'", "amount": "13370000000000000000000" } } } }' --accountId $NEAR_ACCT --amount 1
# near call $DAO_ACCOUNT_ID add_proposal '{"proposal": { "description": "Sample payment", "kind": { "Transfer": { "token_id": "", "receiver_id": "'$NEAR_ACCT'", "amount": "1000000000000000000000000" } } } }' --accountId $NEAR_ACCT --amount 1
# near call $DAO_ACCOUNT_ID add_proposal '{"proposal": { "description": "Sample payment", "kind": { "Transfer": { "token_id": "", "receiver_id": "'$NEAR_ACCT'", "amount": "2000000000000000000000000" } } } }' --accountId $NEAR_ACCT --amount 1
# approve some, leave some
near call $DAO_ACCOUNT_ID act_proposal '{"id": 0, "action" :"VoteApprove"}' --accountId $NEAR_ACCT  --gas $MAX_GAS
# near call $DAO_ACCOUNT_ID act_proposal '{"id": 1, "action" :"VoteApprove"}' --accountId $NEAR_ACCT  --gas $MAX_GAS
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
#### Get DAO v3 code data & store it in factory
#### --------------------------------------------
# Store the code data
V3_BYTES='cat sputnikdao2/res/sputnikdao2.wasm | base64'
near call $FACTORY_ACCOUNT_ID store $(eval "$V3_BYTES") --base64 --accountId $FACTORY_ACCOUNT_ID --gas $MAX_GAS --amount 6 > v3_code_hash_result.txt

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
#### --------------------------------------------


# #### --------------------------------------------
# cleanup local files!
# #### --------------------------------------------
rm sputnikdao2_original.wasm
rm sputnikdao_factory2_original.wasm
rm v2_code_hash_result.txt
rm v3_code_hash_result.txt

echo "Mainnet Factory Deploy & Test Complete"
