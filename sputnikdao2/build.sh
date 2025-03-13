#!/bin/bash
set -e

cargo near build non-reproducible-wasm --no-abi
cp ../target/near/sputnikdao2/sputnikdao2.wasm ./res/
