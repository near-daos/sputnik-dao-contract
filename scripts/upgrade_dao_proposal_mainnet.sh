#!/bin/bash
set -e

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
COMMIT_V3=640495ba572345ca356376989738fbd5462e1ff8
V3_CODE_HASH=783vth3Fg8MBBGGFmRqrytQCWBpYzUcmHoCq4Mo8QqF5

# IMPORTANT!!!!!!!!!!!!!!!!!!!!!!!!!!!
# Change this to YOUR dao
DAO_ACCOUNT_ID=sputnikdao-.$FACTORY_ACCOUNT_ID
# ALSO!!!!!!!!!!!!
# CHANGE ALL THE proposal IDs!!!!! Your DAO could have other proposals, you need to change to use the next ID


#### --------------------------------------------
#### Sanity check the new metadata & DAO
#### --------------------------------------------
near view $FACTORY_ACCOUNT_ID get_contracts_metadata
#### --------------------------------------------


#### --------------------------------------------
#### Upgradeable DAO Proposal
#### --------------------------------------------
# 783vth3Fg8MBBGGFmRqrytQCWBpYzUcmHoCq4Mo8QqF5

UPGRADE_PROPOSAL_ARGS=`echo '{"code_hash":"783vth3Fg8MBBGGFmRqrytQCWBpYzUcmHoCq4Mo8QqF5"}' | base64`
# propose UpgradeSelf using the code_hash from store_blob
near call $DAO_ACCOUNT_ID add_proposal '{
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
near call $DAO_ACCOUNT_ID act_proposal '{"id": 1, "action" :"VoteApprove"}' --accountId $NEAR_ACCT  --gas $MAX_GAS
# quick check all is good
near view $DAO_ACCOUNT_ID get_proposal '{"id": 1}'
#### --------------------------------------------


#### --------------------------------------------
#### Upgrade a v2 DAO to v3
#### --------------------------------------------
# 783vth3Fg8MBBGGFmRqrytQCWBpYzUcmHoCq4Mo8QqF5
echo "Upgrade V3 CODE HASH: $V3_CODE_HASH"
# some sample payouts
near call $DAO_ACCOUNT_ID add_proposal '{"proposal": { "description": "Upgrade to v3", "kind": { "UpgradeSelf": { "hash": "'$V3_CODE_HASH'" } } } }' --accountId $NEAR_ACCT --amount 1
# approve some, leave some
near call $DAO_ACCOUNT_ID act_proposal '{"id": 2, "action" :"VoteApprove"}' --accountId $NEAR_ACCT  --gas $MAX_GAS
# quick check all is good
near view $DAO_ACCOUNT_ID get_proposal '{"id": 2}'
#### --------------------------------------------

#### --------------------------------------------
#### Remove cached blob DAO Proposal
#### --------------------------------------------
# 783vth3Fg8MBBGGFmRqrytQCWBpYzUcmHoCq4Mo8QqF5

REMOVE_PROPOSAL_ARGS=`echo '{"code_hash":"783vth3Fg8MBBGGFmRqrytQCWBpYzUcmHoCq4Mo8QqF5"}' | base64`
# propose UpgradeSelf using the code_hash from store_blob
near call $DAO_ACCOUNT_ID add_proposal '{
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
near call $DAO_ACCOUNT_ID act_proposal '{"id": 3, "action" :"VoteApprove"}' --accountId $NEAR_ACCT  --gas $MAX_GAS
# quick check all is good
near view $DAO_ACCOUNT_ID get_proposal '{"id": 3}'
#### --------------------------------------------

echo "Mainnet DAO Upgrade Complete"
