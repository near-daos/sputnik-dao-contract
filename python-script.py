#!/usr/local/bin/python3.10
import base58
import base64
import hashlib
import json
import subprocess

subprocess.run(["./build.sh"])

MASTER_ACCOUNT = "ctindogaru4.testnet"
FACTORY_ACCOUNT = f"sputnikdao-factory2.{MASTER_ACCOUNT}"
SPUTNIK_REPO_PATH = "/Users/constantindogaru/near-protocol/sputnik-dao-contract"

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

wasm_contract = b""
with open(f"{SPUTNIK_REPO_PATH}/sputnikdao2/res/sputnikdao2.wasm",
          "rb") as wasm_file:
    wasm_contract = wasm_file.read()  # read entire file as bytes

wasm_contract_in_base64 = base64.b64encode(wasm_contract)
hash_in_hex = hashlib.sha256(wasm_contract).hexdigest()
hash_in_bytes = bytes.fromhex(hash_in_hex)
hash_in_base58 = base58.b58encode(hash_in_bytes).decode("UTF-8")

return_value = subprocess.run([
    "near", "call", FACTORY_ACCOUNT, "store", wasm_contract_in_base64,
    "--base64", "--accountId", FACTORY_ACCOUNT
],
                              capture_output=True,
                              text=True).stdout.splitlines()[-1].strip("'")
assert return_value == hash_in_base58  # it means that the contract was stored successfully

params = json.dumps({"code_hash": hash_in_base58})
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
assert latest_hash == hash_in_base58

params = json.dumps({"code_hash": hash_in_base58})
subprocess.run([
    "near", "call", FACTORY_ACCOUNT, "delete_contract", params, "--accountId",
    FACTORY_ACCOUNT
])
