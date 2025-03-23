#!/bin/bash
set -e

cargo near build reproducible-wasm
cp ../target/near/sputnik_staking/sputnik_staking.wasm ./res/
