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
export DAO_ACCOUNT_ID=genesis.$FACTORY_ACCOUNT_ID
export MAX_GAS=300000000000000
export GAS_100_TGAS=100000000000000
export GAS_150_TGAS=150000000000000
BOND_AMOUNT=1
BYTE_STORAGE_COST=6000000000000000000000000
COMMIT_V2=c2cf1553b070d04eed8f659571440b27d398c588
V2_CODE_HASH=8RMeZ5cXDap6TENxaJKtigRYf3n139iHmTRe8ZUNey6N
COMMIT_V2A=TBD
V2A_CODE_HASH=TBD
COMMIT_V3=TBD
V3_CODE_HASH=TBD


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
near call $FACTORY_ACCOUNT_ID store_contract_metadata '{"code_hash": "'$V2_CODE_HASH'", "metadata": {"version": [2,0], "commit_id": "'$COMMIT_V2'"}, "set_default": false}' --accountId $FACTORY_ACCOUNT_ID
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
#### Sanity check the new metadata
#### --------------------------------------------
near view $FACTORY_ACCOUNT_ID get_contracts_metadata
near view $FACTORY_ACCOUNT_ID get_dao_list
#### --------------------------------------------


# #### --------------------------------------------
# cleanup local files!
# #### --------------------------------------------
rm sputnikdao2_original.wasm
rm sputnikdao_factory2_original.wasm
rm v2_code_hash_result.txt
rm v2a_code_hash_result.txt
rm v3_code_hash_result.txt

echo "TESTNET: Factory Deploy Complete"