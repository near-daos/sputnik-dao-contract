#!/bin/bash
set -e

cargo near build non-reproducible-wasm --no-abi
cp ../target/near/sputnik_staking/sputnik_staking.wasm ./res/
