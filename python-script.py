#!/usr/local/bin/python3.10
import base58
import base64
import hashlib
import json
import subprocess

# !!! 1. BEFORE RUNNING THIS SCRIPT YOU NEED TO DELETE THE CHANGE IN views.rs AND THEN MANUALLY RUN ./build.sh
# !!! 2. AFTER THE RUN, CHANGE sputnikdao2/res/sputnikdao2.wasm to sputnikdao2/res/old_sputnikdao2.wasm
# !!! 3. REVERT THE CHANGE IN views.rs
# !!! 4. ADDRESS THE OTHER "!!!" COMMENTS AND ONLY AFTER THAT RUN ./python-script.py

############################################ SETUP ############################################

subprocess.run(["./build.sh"])

# !!! replace SPUTNIK_REPO_PATH and MASTER_ACCOUNT with your owns
SPUTNIK_REPO_PATH = "/Users/constantindogaru/near-protocol/sputnik-dao-contract"
MASTER_ACCOUNT = "ctindogaru6.testnet"
FACTORY_ACCOUNT = f"sputnikdao-factory2.{MASTER_ACCOUNT}"
# !!! change DAO_NAME every time you run the script
DAO_NAME = "dao2"
DAO_ACCOUNT = f"{DAO_NAME}.{FACTORY_ACCOUNT}"

old_wasm_contract = b""
with open(f"{SPUTNIK_REPO_PATH}/sputnikdao2/res/old_sputnikdao2.wasm",
          "rb") as wasm_file:
    old_wasm_contract = wasm_file.read()  # read entire file as bytes

OLD_WASM_CONTRACT_IN_BASE64 = base64.b64encode(old_wasm_contract)
OLD_HASH_IN_HEX = hashlib.sha256(old_wasm_contract).hexdigest()
OLD_HASH_IN_BYTES = bytes.fromhex(OLD_HASH_IN_HEX)
OLD_HASH_IN_BASE58 = base58.b58encode(OLD_HASH_IN_BYTES).decode("UTF-8")

# Create an account for the factory itself
subprocess.run([
    "near", "create-account", FACTORY_ACCOUNT, "--masterAccount",
    MASTER_ACCOUNT, "--initialBalance", "25"
])

# Deploy the factory contract to that account
init_args = json.dumps({})
subprocess.run([
    "near", "deploy", FACTORY_ACCOUNT, "--wasmFile",
    f"{SPUTNIK_REPO_PATH}/sputnikdao-factory2/res/sputnikdao_factory2.wasm",
    "--initFunction", "new", "--initArgs", init_args
])

######################################## TESTING CREATE ########################################

# Store the DAO contract code inside the factory
return_hash = subprocess.run([
    "near", "call", FACTORY_ACCOUNT, "store", OLD_WASM_CONTRACT_IN_BASE64,
    "--base64", "--accountId", FACTORY_ACCOUNT
],
                             capture_output=True,
                             text=True).stdout.splitlines()[-1].strip("'")
assert return_hash == OLD_HASH_IN_BASE58  # it means that the contract was stored successfully

# Get the DAO contract code from the factory
params = json.dumps({"code_hash": OLD_HASH_IN_BASE58})
return_code = subprocess.run([
    "near", "call", FACTORY_ACCOUNT, "get_code", params, "--accountId",
    FACTORY_ACCOUNT
],
                             capture_output=True,
                             text=True).stdout.splitlines()[-1].strip("'")
# TODO: verify return_code

# Save the hash code associated with the DAO contract code inside the factory
params = json.dumps({"code_hash": OLD_HASH_IN_BASE58})
subprocess.run([
    "near", "call", FACTORY_ACCOUNT, "set_code_hash", params, "--accountId",
    FACTORY_ACCOUNT
])

# Get the hash code from the factory
latest_hash = subprocess.run([
    "near", "call", FACTORY_ACCOUNT, "get_latest_code_hash", "--accountId",
    FACTORY_ACCOUNT
],
                             capture_output=True,
                             text=True).stdout.splitlines()[-1].strip("'")
assert latest_hash == OLD_HASH_IN_BASE58

# Create a new DAO
args_param = json.dumps({
    "config": {
        "name": DAO_NAME,
        "purpose": "testing",
        "metadata": ""
    },
    "policy": [MASTER_ACCOUNT]
}).encode("UTF-8")
params = json.dumps({
    "name": DAO_NAME,
    "args": base64.b64encode(args_param).decode("UTF-8")
})
subprocess.run([
    "near", "call", FACTORY_ACCOUNT, "create", params, "--accountId",
    FACTORY_ACCOUNT, "--gas", "300000000000000", "--amount", "10"
])

# Get the list of DAOs
dao_list = subprocess.run([
    "near", "call", FACTORY_ACCOUNT, "get_dao_list", "--accountId",
    FACTORY_ACCOUNT
],
                          capture_output=True,
                          text=True).stdout.splitlines()[-1].strip("'")
assert dao_list == f"[ '{DAO_ACCOUNT}' ]"

# Get last proposal id from the DAO
last_proposal_id = subprocess.run([
    "near", "call", DAO_ACCOUNT, "get_last_proposal_id", "--accountId",
    FACTORY_ACCOUNT
],
                                  capture_output=True,
                                  text=True).stdout.splitlines()[-1]
assert last_proposal_id == "0"

######################################## TESTING UPGRADE ########################################

new_wasm_contract = b""
with open(f"{SPUTNIK_REPO_PATH}/sputnikdao2/res/sputnikdao2.wasm",
          "rb") as wasm_file:
    new_wasm_contract = wasm_file.read()  # read entire file as bytes

NEW_WASM_CONTRACT_IN_BASE64 = base64.b64encode(new_wasm_contract)
NEW_HASH_IN_HEX = hashlib.sha256(new_wasm_contract).hexdigest()
NEW_HASH_IN_BYTES = bytes.fromhex(NEW_HASH_IN_HEX)
NEW_HASH_IN_BASE58 = base58.b58encode(NEW_HASH_IN_BYTES).decode("UTF-8")

# Store the DAO contract code inside the factory
return_hash = subprocess.run([
    "near", "call", FACTORY_ACCOUNT, "store", NEW_WASM_CONTRACT_IN_BASE64,
    "--base64", "--accountId", FACTORY_ACCOUNT
],
                             capture_output=True,
                             text=True).stdout.splitlines()[-1].strip("'")
assert return_hash == NEW_HASH_IN_BASE58  # it means that the contract was stored successfully

# Get the DAO contract code from the factory
params = json.dumps({"code_hash": NEW_HASH_IN_BASE58})
return_code = subprocess.run([
    "near", "call", FACTORY_ACCOUNT, "get_code", params, "--accountId",
    FACTORY_ACCOUNT
],
                             capture_output=True,
                             text=True).stdout.splitlines()[-1].strip("'")
# TODO: verify return_code

# Save the hash code associated with the DAO contract code inside the factory
params = json.dumps({"code_hash": NEW_HASH_IN_BASE58})
subprocess.run([
    "near", "call", FACTORY_ACCOUNT, "set_code_hash", params, "--accountId",
    FACTORY_ACCOUNT
])

# Get the hash code from the factory
latest_hash = subprocess.run([
    "near", "call", FACTORY_ACCOUNT, "get_latest_code_hash", "--accountId",
    FACTORY_ACCOUNT
],
                             capture_output=True,
                             text=True).stdout.splitlines()[-1].strip("'")
assert latest_hash == NEW_HASH_IN_BASE58

# Upgrade the DAO
params = json.dumps({"account_id": DAO_ACCOUNT})
subprocess.run([
    "near", "call", FACTORY_ACCOUNT, "upgrade", params, "--accountId",
    FACTORY_ACCOUNT, "--gas", "300000000000000"
])

# Call the testing method that's only available in the new contract code
last_proposal_id = subprocess.run([
    "near", "call", DAO_ACCOUNT, "get_10_for_testing", "--accountId",
    FACTORY_ACCOUNT
],
                                  capture_output=True,
                                  text=True).stdout.splitlines()[-1]
assert last_proposal_id == "10"

############################################ CLEAN-UP ############################################

# Delete the old DAO contract code from the factory
params = json.dumps({"code_hash": OLD_HASH_IN_BASE58})
subprocess.run([
    "near", "call", FACTORY_ACCOUNT, "delete_contract", params, "--accountId",
    FACTORY_ACCOUNT
])

# Delete the new DAO contract code from the factory
params = json.dumps({"code_hash": NEW_HASH_IN_BASE58})
subprocess.run([
    "near", "call", FACTORY_ACCOUNT, "delete_contract", params, "--accountId",
    FACTORY_ACCOUNT
])

# Delete the account associated with the factory
subprocess.run(["near", "delete", FACTORY_ACCOUNT, MASTER_ACCOUNT])
