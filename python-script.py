#!/usr/local/bin/python3.10
import base58
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

# def hex_str_to_u8_arr(hash_str: str) -> list[int]:
#     u8_array = []
#     for i in range(0, len(hash_str), 2):
#         hex_char = hash_str[i:i + 2]
#         u8_array.append(int(hex_char, 16))
#     return u8_array

subprocess.run([
    "wasm2wat", f"{SPUTNIK_REPO_PATH}/sputnikdao2/res/sputnikdao2.wasm", "-o",
    f"{SPUTNIK_REPO_PATH}/sputnikdao2/res/sputnikdao2.wat"
])
with open(f"{SPUTNIK_REPO_PATH}/sputnikdao2/res/sputnikdao2.wat",
          "r") as wat_file:
    # TODO: wat_contract is too long for near cli
    wat_contract = wat_file.read()  # read entire file as text
    params = json.dumps({"input": wat_contract})
    subprocess.run([
        "near", "call", FACTORY_ACCOUNT, "store", params, "--accountId",
        FACTORY_ACCOUNT
    ])

# with open(f"{SPUTNIK_REPO_PATH}/sputnikdao2/res/sputnikdao2.wasm",
#           "rb") as wasm_file:
#     wasm_contract = wasm_file.read()  # read entire file as bytes

#     hash_in_hex = hashlib.sha256(wasm_contract).hexdigest()
#     hash_in_bytes = bytes.fromhex(hash_in_hex)
#     hash_in_base58 = base58.b58encode(hash_in_bytes).decode("UTF-8")

#     params = json.dumps({"code_hash": hash_in_base58})
#     subprocess.run([
#         "near", "call", FACTORY_ACCOUNT, "set_code_hash", params,
#         "--accountId", FACTORY_ACCOUNT
#     ])
