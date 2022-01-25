#!/bin/sh
set -e
# Change these to your account ids
./build.sh
export CONTRACT_ID=sputnikdao2.md4ire.testnet
export CONTRACT_PARENT=md4ire.testnet

# Redo account (if contract already exists)
set +e
near delete $CONTRACT_ID $CONTRACT_PARENT 2> /dev/null # Ignore errors
set -e
near create-account $CONTRACT_ID --masterAccount $CONTRACT_PARENT

# Set up
near deploy $CONTRACT_ID --wasmFile res/sputnikdao2.wasm
export COUNCIL='["'$CONTRACT_ID'"]'
near call $CONTRACT_ID new '{"config": {"name": "genesis2", "purpose": "test", "metadata": ""}, "policy": '$COUNCIL'}' --accountId $CONTRACT_ID

# Add proposal for a Transfer kind that pays out 19 NEAR
near call $CONTRACT_ID add_proposal '{"proposal": {"description": "test bounty", "kind": {"AddBounty": {"bounty": {"description": "do the thing", "amount": "19000000000000000000000000", "times": 3, "max_deadline": "1925376849430593581"}}}}}' --accountId $CONTRACT_PARENT --amount 1

# Show error when a user tries to vote along with log
near call $CONTRACT_ID act_proposal '{"id": 0, "action": "VoteApprove"}' --accountId $CONTRACT_ID

# Someone claims bounty
near call $CONTRACT_ID bounty_claim '{"id": 0, "deadline": "1925376849430593581"}' --accountId $CONTRACT_PARENT --amount 1

# Show bounty claims
near view $CONTRACT_ID get_bounty_claims '{"account_id": "'$CONTRACT_PARENT'"}'

# Call bounty_done
near call $CONTRACT_ID bounty_done '{"id": 0, "description": "was not even hard. ez"}'  --accountId $CONTRACT_PARENT --amount 1
# Add BountyDone proposal done
# bounty_done adds proposal BountyDone itself
# near call $CONTRACT_ID add_proposal '{"proposal": {"description": "test bounty done", "kind": {"BountyDone": {"bounty_id": 0, "receiver_id": "'$CONTRACT_PARENT'"}}}}' --accountId $CONTRACT_PARENT --amount 1

# Vote it in
near call $CONTRACT_ID act_proposal '{"id": 1, "action": "VoteApprove"}' --accountId $CONTRACT_ID

# See how many now.
near view $CONTRACT_ID get_bounty_claims '{"account_id": "'$CONTRACT_PARENT'"}'