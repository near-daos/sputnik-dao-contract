# Upgrading Smart Contracts in Sputnik - step-by-step walkthrough

## Background

Sometimes, the code inside a smart contract can change as a result of adding new features or improving existing functionality. When that happens, to take advantage of the new contract code, we must re-deploy the smart contract to the mainnet. This should be done in a very responsible manner, especially in Sputnik, since it contains sensitive information and it has more than $1M Total Value Locked inside the smart contracts.

There are two important concepts you need to know about Sputnik. There is a *mother* called the factory and her *children* called the DAOs.
- the factory aka *the mother*: it is considered to be the mother of all the DAOs, since it's responsible for creating them. A new child (DAO) is born when calling the `create` method on the mother (factory) contract (check `sputnikdao-factory2/src/lib.rs` for the factory code).
- the DAOs aka *the children*: once created, they become independent from their mother and they have their own set of rules and policies that help them with self-governance (check `sputnikdao2/src/lib.rs` for the DAO code).

***

## History of Sputnik - Searching through the archives:

### Sputnik V1 smart contracts
**Factory**: 
- the factory code can be found inside `sputnikdao-factory/src`
- the factory code is deployed on mainnet on `sputnikdao.near` account
- the factory code was deployed on mainnet for the first time on [January 11, 2021 at 2:29:30pm](https://explorer.mainnet.near.org/transactions/Cxx9pPfrc12N79W8oREPYA2b1CwU6JKitXwAem9C4jHB) and the second time on [January 20, 2021 at 1:31:05pm
](https://explorer.mainnet.near.org/transactions/653FzhtCKrhsxR5nmzHcuP3AnL5eLQYFyhU6tU4qMydg)
- the factory code is deployed on testnet on `sputnik-v1.testnet` account

**DAOs**:
- the DAOs code can be found inside `sputnikdao/src`
- the DAO code is deployed to several accounts that have the following format: `<dao_name>.sputnikdao.near` for mainnet and `<dao_name>.sputnik-v1.testnet` for testnet
- see all the DAOs that live on mainnet by calling `near view sputnikdao.near get_dao_list`
- see all the DAOs that live on testnet by calling `near view sputnik-v1.testnet get_dao_list`

### Sputnik v2 smart contracts
**Factory**: 
- the factory code can be found inside `sputnikdao-factory2/src`
- the factory code is deployed on mainnet on `sputnik-dao.near` account
- the factory code was deployed only once on mainnet on [June 01, 2021 at 6:17:46pm](https://explorer.mainnet.near.org/transactions/HZZpqMjbhCPWF8n5DXLMq5WtpmASfXuDLtYF6vNsiD9U)
- the factory code is deployed on testnet on `sputnikv2.testnet` account

**DAOs**:
- the DAOs code can be found inside `sputnikdao2/src`
- the DAO code is deployed to several accounts that have the following format: `<dao_name>.sputnik-dao.near` for mainnet and `<dao_name>.sputnikv2.testnet` for testnet
- see all the DAOs that live on mainnet by calling `near view sputnik-dao.near get_dao_list`
- see all the DAOs that live on testnet by calling `near view sputnik-v1.testnet get_dao_list`

### UI DApps built on top of the Sputnik smart contracts (from oldest to newest)
- https://old.sputnik.fund/ -> it uses V1 smart contracts
- https://www.sputnik.fund/ -> it uses V1 smart contracts
- https://v2.sputnik.fund/ -> it uses v2 smart contracts
- https://astrodao.com/ -> it uses v2 smart contracts

***

## Introducing Sputnik v3 smart contracts

The biggest advantage of v3 smart contracts is introducing an easy way for the DAO to upgrade to a new version of the code so it can take full advantage of the new features/performance improvements/bug fixes.  

Since this is the first time that the factory and the DAO are being upgraded and upgrading smart contracts is a very sensitive topic, everything must be done with due diligence.

### v3 Release Plan

[1 Upgrade the factory from v2 to v3 and then set up the default code for the DAO to be v2.](#1-upgrade-the-factory-from-v2-to-v3)  
[2 After we get enough confidence using factory v3 and DAO v2, change the default code for the DAO from v2 to v3.](#2-change-dao-default-code-from-v2-to-v3)  
[3 Existing DAOs will need to migrate from v2 to v3.](#3-migrate-dao-from-v2-to-v3)  

### 1 Upgrade the factory from v2 to v3

This should be done in the following order:
- [testnet - using a personal account](#1-1-using-personal-account-on-testnet)
- [testnet - using the official testnet factory account](#1-2-using-official-account-on-testnet)
- [mainnet - using the official mainnet factory account](#1-3-using-official-account-on-mainnet)

___

### 1 1 Using personal account on testnet

***1. Create a new NEAR account for the factory:***

```bash
near create-account sputnik-factory.ctindogaru.testnet --masterAccount ctindogaru.testnet --initialBalance 100
```

***2. Deploy the factory code:***
```bash
./build.sh
```
```bash
near deploy sputnik-factory.ctindogaru.testnet sputnikdao-factory2/res/sputnikdao_factory2.wasm
```

***3. Init the factory:***
```bash
near call sputnik-factory.ctindogaru.testnet new '{}' --accountId sputnik-factory.ctindogaru.testnet --gas 100000000000000
```

***4. Download the current `wasm` code used for creating new DAOs:***

Go to [Astro DAO Testnet](https://testnet.app.astrodao.com/all/daos) and pick any dao from the list to download its wasm code. In this example, I have picked `thegame.sputnikv2.testnet`.
```bash
http --json post https://rpc.testnet.near.org jsonrpc=2.0 id=dontcare method=query \
params:='{"request_type":"view_code","finality":"final","account_id":"thegame.sputnikv2.testnet"}' \
| jq -r .result.code_base64 \
| base64 --decode > dao-code-v2.wasm
```

***5. Use the code downloaded at the previous step and store it inside the factory as the default code used for creating new DAOs:***
```bash
BYTES='cat dao-code-v2.wasm | base64'
```
```bash
near call sputnik-factory.ctindogaru.testnet store $(eval "$BYTES") --base64 --accountId sputnik-factory.ctindogaru.testnet --gas 100000000000000 --amount 10
```

***6. Use the code hash returned from the previous step to store the metadata associated with the code:***
```bash
near call sputnik-factory.ctindogaru.testnet store_contract_metadata '{"code_hash": "ZGdM2TFdQpcXrxPxvq25514EViyi9xBSboetDiB3Uiq", "metadata": {"version": [2, 0], "commit_id": "c2cf1553b070d04eed8f659571440b27d398c588"}, "set_default": true}' --accountId sputnik-factory.ctindogaru.testnet
```

***7. See all the contract versions stored inside the factory:***
```bash
near view sputnik-factory.ctindogaru.testnet get_contracts_metadata
```
2 versions should be displayed. The one that got created on init and the one that you stored in the previous step.

***8. Try to create a new DAO from the factory - using NEAR CLI:***
```bash
export COUNCIL='["ctindogaru.testnet"]'
```
```bash
export ARGS=`echo '{"config": {"name": "ctindogaru-dao", "purpose": "ctindogaru DAO", "metadata":""}, "policy": '$COUNCIL'}' | base64`
```
```bash
near call sputnik-factory.ctindogaru.testnet create "{\"name\": \"ctindogaru-dao\", \"args\": \"$ARGS\"}" --accountId sputnik-factory.ctindogaru.testnet --gas 150000000000000 --amount 10
```

***9. See all the DAOs created by the factory:***
```bash
near view sputnik-factory.ctindogaru.testnet get_dao_list
```
The DAO created in the previous step should be displayed here.

***10. Try to interact with the DAO and make sure everything works:***
```bash
near view ctindogaru-dao.sputnik-factory.ctindogaru.testnet get_available_amount
```

You can now jump to section 3.1 for upgrading the dao from v2 to v3.

___

### 1 2 Using official account on testnet

***1. Upgrade the factory code:***
```bash
./build.sh
```
```bash
near deploy sputnikv2.testnet sputnikdao-factory2/res/sputnikdao_factory2.wasm
```

***2. Download the current `wasm` code used for creating new DAOs:***

```bash
near view sputnikv2.testnet get_dao_list
```
Now pick any dao from the returned list and use it to download the wasm code:
```bash
http --json post https://rpc.testnet.near.org jsonrpc=2.0 id=dontcare method=query \
params:='{"request_type":"view_code","finality":"final","account_id":"thegame.sputnikv2.testnet"}' \
| jq -r .result.code_base64 \
| base64 --decode > dao-code-v2.wasm
```

***3. Use the code downloaded at the previous step and store it inside the factory as the default code used for creating new DAOs:***
```bash
BYTES='cat dao-code-v2.wasm | base64'
```
```bash
near call sputnikv2.testnet store $(eval "$BYTES") --base64 --accountId sputnikv2.testnet --gas 100000000000000 --amount 10
```

***4. Use the code hash returned from the previous step to store the metadata associated with the code:***
```bash
near call sputnikv2.testnet store_contract_metadata '{"code_hash": "ZGdM2TFdQpcXrxPxvq25514EViyi9xBSboetDiB3Uiq", "metadata": {"version": [2, 0], "commit_id": "c2cf1553b070d04eed8f659571440b27d398c588"}, "set_default": true}' --accountId sputnikv2.testnet
```

***5. See all the contract versions stored inside the factory:***
```bash
near view sputnikv2.testnet get_contracts_metadata
```
Only the version stored in the previous step should be displayed.

***6. Try to create a new DAO using the new factory - using Astro DAO:***

Go to https://testnet.app.astrodao.com/all/daos and try to create a new DAO from the UI. It should use the new version of the factory code.  

Please note that the DAO itself is still v2. The only difference is that the DAO gets created through the v3 version of the factory.

___

### 1 3 Using official account on mainnet

The process is very similar with 1.2.

___

### 2 Change DAO default code from v2 to v3

After a few weeks of running factory v3 + DAO v2, it's time to step up the game and upgrade the default DAO version to v3.

This should be done in the following order:
- [testnet - using the official testnet factory account](#2-1-using-official-account-on-testnet)
- [mainnet - using the official mainnet factory account](#2-2-using-official-account-on-mainnet)

Assumptions:
- the DAO v3 code will be a snapshot of the code from commit id `16b3bac754d18bbf88b1738cb0d6508cfd408bb2`.

___

### 2 1 Using official account on testnet

***1. Checkout to the right commit id***
```bash
git checkout 16b3bac754d18bbf88b1738cb0d6508cfd408bb2
```

***2. Build the DAO v3 code:***
```bash
./build.sh
```

***3. Use the code built at the previous step and store it inside the factory as the default code used for creating new DAOs:***
```bash
BYTES='cat sputnikdao2/res/sputnikdao2.wasm | base64'
```
```bash
near call sputnikv2.testnet store $(eval "$BYTES") --base64 --accountId sputnikv2.testnet --gas 100000000000000 --amount 10
```

***4. Use the code hash returned from the previous step to store the metadata associated with the code:***
```bash
near call sputnikv2.testnet store_contract_metadata '{"code_hash": "6SQymHtmezR3u9zAtLBQdb8RWCXxwxnigqSH2mRTxecB", "metadata": {"version": [3, 0], "commit_id": "16b3bac754d18bbf88b1738cb0d6508cfd408bb2"}, "set_default": true}' --accountId sputnikv2.testnet
```

***5. See all the contract versions stored inside the factory:***
```bash
near view sputnikv2.testnet get_contracts_metadata
```
2 versions should be displayed:
- v2 with commit id `c2cf1553b070d04eed8f659571440b27d398c588` and hash `ZGdM2TFdQpcXrxPxvq25514EViyi9xBSboetDiB3Uiq`
- v3 with commit id `16b3bac754d18bbf88b1738cb0d6508cfd408bb2` and hash `6SQymHtmezR3u9zAtLBQdb8RWCXxwxnigqSH2mRTxecB`

***6. Try to create a DAO v3 - using Astro DAO:***

Go to https://testnet.app.astrodao.com/all/daos and try to create a new DAO from the UI.  
The DAO that gets created should be a brand new v3 DAO.

___

### 2 2 Using official account on mainnet

The process is very similar with 2.1.

___

### 3 Migrate DAO from v2 to v3

**!!!!!!!!!!! THIS MAY INTRODUCE BACKWARD COMPATIBILITY ISSUES. DO IT ON YOUR OWN RISK !!!!!!!!!!!**

This should be done in the following order:
- [testnet - using a personal account](#3-1-using-personal-account-on-testnet)
- [mainnet - using the official mainnet factory account](#3-2-using-official-account-on-mainnet)

___

### 3 1 Using personal account on testnet

Assumptions:
- we are the trying to upgrade `ctindogaru-dao.sputnik-factory.ctindogaru.testnet` from v2 to v3
- we have the account id `ctindogaru.testnet`

***1. Store the DAO code in your DAO.***

```bash
BYTES='cat sputnikdao2/res/sputnikdao2.wasm | base64'
```

```bash
near call ctindogaru-dao.sputnik-factory.ctindogaru.testnet store_blob $(eval "$BYTES") --base64 --accountId ctindogaru.testnet --gas 100000000000000 --amount 10
```

After running the command from above, you should get the following code hash in return: `6SQymHtmezR3u9zAtLBQdb8RWCXxwxnigqSH2mRTxecB`.  

If your result differs from `6SQymHtmezR3u9zAtLBQdb8RWCXxwxnigqSH2mRTxecB`, **DO NOT** proceed further. Please ask for help in [Telegram](https://t.me/astro_near).

***2. Create an upgrade proposal for your DAO.***

Run the following command:
```bash
near call ctindogaru-dao.sputnik-factory.ctindogaru.testnet add_proposal '{"proposal": {"description": "Upgrade DAO to v3 version", "kind": {"UpgradeSelf": {"hash": "6SQymHtmezR3u9zAtLBQdb8RWCXxwxnigqSH2mRTxecB"}}}}' --accountId ctindogaru.testnet --amount 1
```

The command should return the proposal id which is 0.

***3. Check that the proposal has been created.***

```bash
near view ctindogaru-dao.sputnik-factory.ctindogaru.testnet get_proposal '{"id": 0}'
```

***4. Approve the proposal.***

```bash
near call ctindogaru-dao.sputnik-factory.ctindogaru.testnet act_proposal '{"id": 0, "action": "VoteApprove"}' --accountId ctindogaru.testnet --gas 300000000000000
```

***5. Once the proposal get approved, the upgrade will take place.***

***6. Now that the upgrade is complete, remove the code from your DAO.***

```bash
near call ctindogaru-dao.sputnik-factory.ctindogaru.testnet remove_blob '{"hash": "6SQymHtmezR3u9zAtLBQdb8RWCXxwxnigqSH2mRTxecB"}' --accountId ctindogaru.testnet --gas 100000000000000
```

***7. Congrats! You're now using the DAO v3.***

___

### 3 2 Using official account on mainnet

Assumptions:
- we are the trying to upgrade `amber.sputnik-dao.near` from v2 to v3
- we have the account id `ctindogaru.near`

***1. Open a bash terminal and login to your near account:***

```bash
export NEAR_ENV=mainnet
```

```bash
near login
```

***2. Clone the sputnik repo and go to the v3 snapshot of the code.***

```bash
git clone https://github.com/near-daos/sputnik-dao-contract && cd sputnik-dao-contract
```

```bash
git checkout 16b3bac754d18bbf88b1738cb0d6508cfd408bb2
```

***3. Store the DAO code in your DAO.***

```bash
BYTES='cat sputnikdao2/res/sputnikdao2.wasm | base64'
```

```bash
near call amber.sputnik-dao.near store_blob $(eval "$BYTES") --base64 --accountId ctindogaru.near --gas 100000000000000 --amount 10
```

After running the command from above, you should get the following code hash in return: `6SQymHtmezR3u9zAtLBQdb8RWCXxwxnigqSH2mRTxecB`.  

If your result differs from `6SQymHtmezR3u9zAtLBQdb8RWCXxwxnigqSH2mRTxecB`, **DO NOT** proceed further. Please ask for help in [Telegram](https://t.me/astro_near).

***4. Create an upgrade proposal for your DAO.***

Run the following command:
```bash
near call amber.sputnik-dao.near add_proposal '{"proposal": {"description": "Upgrade DAO to v3 version", "kind": {"UpgradeSelf": {"hash": "6SQymHtmezR3u9zAtLBQdb8RWCXxwxnigqSH2mRTxecB"}}}}' --accountId ctindogaru.near --amount 1
```

***5. Approve the proposal.***

Everyone from the DAO should go to https://app.astrodao.com/dao/amber.sputnik-dao.near/proposals and approve the proposal.

***6. Once the proposal get approved, the upgrade will take place.***

***7. Now that the upgrade is complete, remove the code from your DAO.***

```bash
near call amber.sputnik-dao.near remove_blob '{"hash": "6SQymHtmezR3u9zAtLBQdb8RWCXxwxnigqSH2mRTxecB"}' --accountId ctindogaru.near --gas 100000000000000
```

***8. Congrats! You're now using the DAO v3.***
