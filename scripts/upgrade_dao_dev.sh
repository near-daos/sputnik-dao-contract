#!/bin/bash
# This file is used for starting a fresh set of all contracts & configs
set -e

if [ -d "res" ]; then
  echo ""
else
  mkdir res
fi

cd "`dirname $0`"

if [ -z "$KEEP_NAMES" ]; then
  export RUSTFLAGS='-C link-arg=-s'
else
  export RUSTFLAGS=''
fi

export NEAR_ENV=testnet
export FACTORY=testnet

if [ -z ${NEAR_ACCT+x} ]; then
  export NEAR_ACCT=sputnikv2.$FACTORY
else
  export NEAR_ACCT=$NEAR_ACCT
fi

export WALLET_ACCOUNT_ID=your_council_account.$FACTORY
export FACTORY_ACCOUNT_ID=$NEAR_ACCT
# NOTE: Change this!
export DAO_ACCOUNT_ID=YOUR_DAO_HERE.$FACTORY_ACCOUNT_ID

# Create V3 code & metadata
# NOTE: Change this to the official V3!
V3_CODE_HASH=GUMFKZP6kdLgy3NjKy1EAkn77AfZFLKkj96VAgjmHXeS
near call $DAO_ACCOUNT_ID add_proposal '{"proposal": {"description": "Upgrade DAO to Version 3.0", "kind": {"UpgradeSelf": {"hash": "'$V3_CODE_HASH'"}}}}' --accountId $WALLET_ACCOUNT_ID --amount 1

echo "Dev: Go to https://testnet.app.astrodao.com/all/daos and approve the proposal. Once done, you can verify the DAO is on version 3.0 by running the cmd: `near view $DAO_ACCOUNT_ID get_available_amount`"