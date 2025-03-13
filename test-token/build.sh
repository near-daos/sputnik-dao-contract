#!/bin/bash
set -e

cargo near build non-reproducible-wasm --no-abi
cp ../target/near/test_token/test_token.wasm ./res/
