#!/bin/bash
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
export FACTORY_ACCOUNT_ID=factory_v3_001.$NEAR_ACCT
export STAKING_ACCOUNT_ID=staking_contract_v1_001.$NEAR_ACCT
export TOKEN_ACCOUNT_ID=sample_ft_v3_001.$NEAR_ACCT
# export DAO_ACCOUNT_ID=samples-1651616675.$FACTORY_ACCOUNT_ID
export DAO_ACCOUNT_ID=THIS_WILL_GET_SET_UPON_DEPLOY_BELOW
export USER_ACCOUNT_ID=$NEAR_ACCT
export MAX_GAS=300000000000000
export GAS_100_TGAS=100000000000000
export GAS_150_TGAS=150000000000000

#### --------------------------------------------
#### New Factory for entire test
#### --------------------------------------------
near create-account $FACTORY_ACCOUNT_ID --masterAccount $NEAR_ACCT --initialBalance 80
near create-account $STAKING_ACCOUNT_ID --masterAccount $NEAR_ACCT --initialBalance 10
near create-account $TOKEN_ACCOUNT_ID --masterAccount $NEAR_ACCT --initialBalance 10
#### --------------------------------------------

# Deploy new FT
near deploy --wasmFile test-token/res/fungible_token.wasm --accountId $TOKEN_ACCOUNT_ID
near call $TOKEN_ACCOUNT_ID new '{"owner_id": "'$TOKEN_ACCOUNT_ID'", "total_supply": "1000000000000000", "metadata": { "spec": "ft-1.0.0", "name": "Sample Token Name", "symbol": "SMPL", "decimals": 8 }}' --accountId $TOKEN_ACCOUNT_ID

# Send some tokens to some accounts
near call $TOKEN_ACCOUNT_ID storage_deposit '' --accountId $USER_ACCOUNT_ID --amount 0.00125
near call $TOKEN_ACCOUNT_ID storage_deposit '' --accountId $STAKING_ACCOUNT_ID --amount 0.00125
near view $TOKEN_ACCOUNT_ID ft_balance_of '{"account_id": "'$TOKEN_ACCOUNT_ID'"}'
near view $TOKEN_ACCOUNT_ID ft_balance_of '{"account_id": "'$USER_ACCOUNT_ID'"}'
near call $TOKEN_ACCOUNT_ID ft_transfer '{"receiver_id": "'$USER_ACCOUNT_ID'", "amount": "10000000000000"}' --accountId $TOKEN_ACCOUNT_ID --amount 0.000000000000000000000001
near view $TOKEN_ACCOUNT_ID ft_balance_of '{"account_id": "'$USER_ACCOUNT_ID'"}'

#### --------------------------------------------
#### Deploy factory v3 code data
#### --------------------------------------------
near deploy --wasmFile sputnikdao-factory2/res/sputnikdao_factory2.wasm --accountId $FACTORY_ACCOUNT_ID --initFunction new --initArgs '{}' --initGas $MAX_GAS
#### --------------------------------------------


#### --------------------------------------------
#### Deploy SampleDAO
#### --------------------------------------------
COUNCIL='["'$USER_ACCOUNT_ID'"]'
TIMESTAMP=$(date +"%s")
DAO_NAME=samples-$TIMESTAMP
DAO_ARGS=`echo '{"config": {"name": "'$DAO_NAME'", "purpose": "A DAO that governs samples - Best way to vote for new samples at your local store", "metadata":""}, "policy": '$COUNCIL'}' | base64`
near call $FACTORY_ACCOUNT_ID create "{\"name\": \"$DAO_NAME\", \"args\": \"$DAO_ARGS\"}" --accountId $FACTORY_ACCOUNT_ID --gas $GAS_150_TGAS --amount 8
DAO_ACCOUNT_ID=$DAO_NAME.$FACTORY_ACCOUNT_ID
#### --------------------------------------------

# Deploy staking contract (OWNER MUST BE THE DAO!!!!!!)
near deploy $STAKING_ACCOUNT_ID --wasmFile=sputnik-staking/res/sputnik_staking.wasm --accountId $STAKING_ACCOUNT_ID --initFunction new --initArgs '{"owner_id": "'$DAO_ACCOUNT_ID'","token_id": "'$TOKEN_ACCOUNT_ID'","unstake_period": "3600"}' --force
# near deploy $STAKING_ACCOUNT_ID --wasmFile=sputnik-staking/res/sputnik_staking.wasm --accountId $STAKING_ACCOUNT_ID --force


# Change DAO to use a staking contract
near call $DAO_ACCOUNT_ID add_proposal '{"proposal": { "description": "", "kind": { "SetStakingContract": { "staking_id": "'$STAKING_ACCOUNT_ID'" } } } }' --accountId $USER_ACCOUNT_ID --amount 1
near call $DAO_ACCOUNT_ID act_proposal '{"id": 0, "action" :"VoteApprove"}' --accountId $USER_ACCOUNT_ID  --gas $MAX_GAS
near view $DAO_ACCOUNT_ID get_staking_contract

# # Storage Costs
# near call $STAKING_ACCOUNT_ID storage_unregister '' --accountId $USER_ACCOUNT_ID --gas $MAX_GAS --depositYocto 1
near call $STAKING_ACCOUNT_ID storage_deposit '' --accountId $USER_ACCOUNT_ID --amount 0.01

# NOTE: This assumes you have some FT, and are ready to deposit into the newly deployed staking contract, if you need to create your own FT: https://github.com/near-examples/FT
# Send tokens to the staking contract
near call $TOKEN_ACCOUNT_ID ft_transfer_call '{"receiver_id": "'$STAKING_ACCOUNT_ID'", "amount": "123456789", "msg": ""}' --accountId $USER_ACCOUNT_ID --gas $MAX_GAS --depositYocto 1

# Delegation
near call $STAKING_ACCOUNT_ID delegate '{"account_id": "'$USER_ACCOUNT_ID'", "amount": "123456789"}' --accountId $USER_ACCOUNT_ID --gas $MAX_GAS

# Check user info
near view $STAKING_ACCOUNT_ID get_user '{"account_id": "'$USER_ACCOUNT_ID'"}'


# Make a transfer proposal happen, so we can test token weighting works
PAYOUT_AMT=1000000000000000000000000
near call $DAO_ACCOUNT_ID add_proposal '{"proposal": { "description": "Payout", "kind": { "Transfer": { "token_id": "", "receiver_id": "'$USER_ACCOUNT_ID'", "amount": "'$PAYOUT_AMT'" } } } }' --accountId $USER_ACCOUNT_ID --amount 1
near call $DAO_ACCOUNT_ID act_proposal '{"id": 1, "action" :"VoteApprove"}' --accountId $USER_ACCOUNT_ID  --gas $MAX_GAS

# Undelegation
near call $STAKING_ACCOUNT_ID undelegate '{"account_id": "'$USER_ACCOUNT_ID'", "amount": "123456789"}' --accountId $USER_ACCOUNT_ID --gas $MAX_GAS

# Withdraw tokens from staking contract
near call $STAKING_ACCOUNT_ID withdraw '{"amount": "123456789"}' --accountId $USER_ACCOUNT_ID --gas $MAX_GAS

# Check final user info
near view $STAKING_ACCOUNT_ID get_user '{"account_id": "'$USER_ACCOUNT_ID'"}'
near view $TOKEN_ACCOUNT_ID ft_balance_of '{"account_id": "'$USER_ACCOUNT_ID'"}'

echo "Token Staking Test Complete"