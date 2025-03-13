#!/bin/bash
set -e

cargo near build non-reproducible-wasm --no-abi
cp ../target/near/sputnikdao_factory2/sputnikdao_factory2.wasm ./res/