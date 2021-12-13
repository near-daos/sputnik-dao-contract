#!/usr/local/bin/python3.10
import base58
import base64
import hashlib
import json
import subprocess

subprocess.run(["./build.sh"])

SPUTNIK_REPO_PATH = "/Users/constantindogaru/near-protocol/sputnik-dao-contract"
MASTER_ACCOUNT = "ctindogaru4.testnet"
FACTORY_ACCOUNT = f"sputnikdao-factory2.{MASTER_ACCOUNT}"

wasm_contract = b""
with open(f"{SPUTNIK_REPO_PATH}/sputnikdao2/res/sputnikdao2.wasm",
          "rb") as wasm_file:
    wasm_contract = wasm_file.read()  # read entire file as bytes

WASM_CONTRACT_IN_BASE64 = base64.b64encode(wasm_contract)
HASH_IN_HEX = hashlib.sha256(wasm_contract).hexdigest()
HASH_IN_BYTES = bytes.fromhex(HASH_IN_HEX)
HASH_IN_BASE58 = base58.b58encode(HASH_IN_BYTES).decode("UTF-8")

subprocess.run(["near", "delete", FACTORY_ACCOUNT, MASTER_ACCOUNT])
subprocess.run([
    "near", "create-account", FACTORY_ACCOUNT, "--masterAccount",
    MASTER_ACCOUNT, "--initialBalance", "10"
])

init_args = json.dumps({})
subprocess.run([
    "near", "deploy", FACTORY_ACCOUNT, "--wasmFile",
    f"{SPUTNIK_REPO_PATH}/sputnikdao-factory2/res/sputnikdao_factory2.wasm",
    "--initFunction", "new", "--initArgs", init_args
])

return_hash = subprocess.run([
    "near", "call", FACTORY_ACCOUNT, "store", WASM_CONTRACT_IN_BASE64,
    "--base64", "--accountId", FACTORY_ACCOUNT
],
                             capture_output=True,
                             text=True).stdout.splitlines()[-1].strip("'")
assert return_hash == HASH_IN_BASE58  # it means that the contract was stored successfully

params = json.dumps({"code_hash": HASH_IN_BASE58})
return_code = subprocess.run([
    "near", "call", FACTORY_ACCOUNT, "get_code", params, "--accountId",
    FACTORY_ACCOUNT
],
                             capture_output=True,
                             text=True).stdout.splitlines()[-1].strip("'")
# TODO: verify return_code

params = json.dumps({"code_hash": HASH_IN_BASE58})
subprocess.run([
    "near", "call", FACTORY_ACCOUNT, "set_code_hash", params, "--accountId",
    FACTORY_ACCOUNT
])

latest_hash = subprocess.run([
    "near", "call", FACTORY_ACCOUNT, "get_latest_code_hash", "--accountId",
    FACTORY_ACCOUNT
],
                             capture_output=True,
                             text=True).stdout.splitlines()[-1].strip("'")
assert latest_hash == HASH_IN_BASE58

params = json.dumps({"code_hash": HASH_IN_BASE58})
subprocess.run([
    "near", "call", FACTORY_ACCOUNT, "delete_contract", params, "--accountId",
    FACTORY_ACCOUNT
])
