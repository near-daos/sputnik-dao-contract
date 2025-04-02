#!/bin/bash
set -e

cargo near build reproducible-wasm
cp ../target/near/test_token/test_token.wasm ./res/
