#!/bin/bash
set -e

cargo near build reproducible-wasm
mkdir -p ./res
cp ../target/near/test_token/test_token.wasm ./res/
