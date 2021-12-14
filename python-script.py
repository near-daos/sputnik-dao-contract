#!/usr/local/bin/python3.10
import base58
import base64
import hashlib
import json
import subprocess

############################################ SETUP ############################################

subprocess.run(["./build.sh"])

# !!! replace SPUTNIK_REPO_PATH and MASTER_ACCOUNT with your owns
SPUTNIK_REPO_PATH = "/Users/constantindogaru/near-protocol/sputnik-dao-contract"
MASTER_ACCOUNT = "ctindogaru5.testnet"
FACTORY_ACCOUNT = f"sputnikdao-factory2.{MASTER_ACCOUNT}"
# !!! change DAO_NAME every time you run the script
DAO_NAME = "dao8"
DAO_ACCOUNT = f"{DAO_NAME}.{FACTORY_ACCOUNT}"

wasm_contract = b""
with open(f"{SPUTNIK_REPO_PATH}/sputnikdao2/res/sputnikdao2.wasm",
          "rb") as wasm_file:
    wasm_contract = wasm_file.read()  # read entire file as bytes

WASM_CONTRACT_IN_BASE64 = base64.b64encode(wasm_contract)
HASH_IN_HEX = hashlib.sha256(wasm_contract).hexdigest()
HASH_IN_BYTES = bytes.fromhex(HASH_IN_HEX)
HASH_IN_BASE58 = base58.b58encode(HASH_IN_BYTES).decode("UTF-8")

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

############################################ TESTING ############################################

# Store the DAO contract code inside the factory
return_hash = subprocess.run([
    "near", "call", FACTORY_ACCOUNT, "store", WASM_CONTRACT_IN_BASE64,
    "--base64", "--accountId", FACTORY_ACCOUNT
],
                             capture_output=True,
                             text=True).stdout.splitlines()[-1].strip("'")
assert return_hash == HASH_IN_BASE58  # it means that the contract was stored successfully

# Get the DAO contract code from the factory
params = json.dumps({"code_hash": HASH_IN_BASE58})
return_code = subprocess.run([
    "near", "call", FACTORY_ACCOUNT, "get_code", params, "--accountId",
    FACTORY_ACCOUNT
],
                             capture_output=True,
                             text=True).stdout.splitlines()[-1].strip("'")
# TODO: verify return_code

# Save the hash code associated with the DAO contract code inside the factory
params = json.dumps({"code_hash": HASH_IN_BASE58})
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
assert latest_hash == HASH_IN_BASE58

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

# Upgrade the DAO
params = json.dumps({"account_id": DAO_ACCOUNT})
subprocess.run([
    "near", "call", FACTORY_ACCOUNT, "upgrade", params, "--accountId",
    FACTORY_ACCOUNT, "--gas", "300000000000000"
])

############################################ CLEAN-UP ############################################

# Delete the DAO contract code from the factory
params = json.dumps({"code_hash": HASH_IN_BASE58})
subprocess.run([
    "near", "call", FACTORY_ACCOUNT, "delete_contract", params, "--accountId",
    FACTORY_ACCOUNT
])

# Delete the account associated with the factory
subprocess.run(["near", "delete", FACTORY_ACCOUNT, MASTER_ACCOUNT])
