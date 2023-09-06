# Sputnik Staking

This is staking contract for Sputnik DAO.

The default version just allows to stake the tokens by users and convert them into "weight" in the Sputnik itself.
Further modifications can be done to allow to leverage the staked token for other functions (providing liquidity for example).


### Token voting

> DAO votes to select some token to become voting token (only can be done once, can't change later).

User flow to vote with selected token:

- Users deposit the desired amount of the token to the separate staking contract defined by the DAO.
- They can then choose who to delegate these tokens. It can be to themselves or to other users to increase their vote weight.
- When users vote for proposals, their vote is weighted by all the delegations to them.
- Undelegating will block delegating / withdrawing until one voting period passes.
- Undelegated tokens can be withdrawn by the user.



## Scripted Flow

NOTE: This is not 100% working, help finalize :)

```bash
export STAKING_ACCOUNT_ID=YOUR_STAKING_CONTRACT.testnet
export DAO_ACCOUNT_ID=YOUR_DAO.sputnikv2.testnet
export TOKEN_ACCOUNT_ID=YOUR_TOKEN_ID.testnet
export USER_ACCOUNT_ID=YOU.testnet
export MAX_GAS=300000000000000

# Deploy staking contract
near deploy $STAKING_ACCOUNT_ID --wasmFile=sputnik-staking/res/sputnik_staking.wasm --accountId $STAKING_ACCOUNT_ID --initFunction new --initArgs '{"owner_id": "'$DAO_ACCOUNT_ID'","token_id": "'$TOKEN_ACCOUNT_ID'","unstake_period": "604800000"}'

# Change DAO to use a staking contract
near call $DAO_ACCOUNT_ID add_proposal '{"proposal": { "description": "", "kind": { "SetStakingContract": { "staking_id": "'$STAKING_ACCOUNT_ID'" } } } }' --accountId $USER_ACCOUNT_ID --amount 1
near call $DAO_ACCOUNT_ID act_proposal '{"id": 0, "action" :"VoteApprove"}' --accountId $USER_ACCOUNT_ID  --gas $MAX_GAS
near view $DAO_ACCOUNT_ID get_staking_contract

# Storage Costs
near call $STAKING_ACCOUNT_ID storage_deposit '{"registration_only": true}' --accountId $STAKER_ACCOUNT_ID --amount 0.01

# NOTE: This assumes you have some FT, and are ready to deposit into the newly deployed staking contract, if you need to create your own FT: https://github.com/near-examples/FT
# Send tokens to the staking contract
near call $TOKEN_ACCOUNT_ID ft_transfer_call '{"sender_id": "'$USER_ACCOUNT_ID'", "amount": "123456789"}' --accountId $USER_ACCOUNT_ID --gas $MAX_GAS

# Delegation
near call $STAKING_ACCOUNT_ID delegate '{"account_id": "'$USER_ACCOUNT_ID'", "amount": "123456789"}' --accountId $USER_ACCOUNT_ID --gas $MAX_GAS

# Check user info
near view $STAKING_ACCOUNT_ID get_user '{"account_id": "'$USER_ACCOUNT_ID'"}'

# Undelegation
near call $STAKING_ACCOUNT_ID undelegate '{"account_id": "'$USER_ACCOUNT_ID'", "amount": "123456789"}' --accountId $USER_ACCOUNT_ID --gas $MAX_GAS

# Withdraw tokens from staking contract
near call $STAKING_ACCOUNT_ID withdraw '{"amount": "123456789"}' --accountId $USER_ACCOUNT_ID --gas $MAX_GAS
```

## ABIs

### Staking Contract :: V1
```
{
  "viewMethods": [
    "ft_total_supply",
    "ft_balance_of",
    "get_user",
    "storage_balance_of"
  ],
  "changeMethods": [
    "new",
    "delegate",
    "undelegate",
    "withdraw",
    "storage_deposit",
    "storage_withdraw",
    "storage_unregister"
  ],
}
```
