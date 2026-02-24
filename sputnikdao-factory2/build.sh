#!/bin/bash
set -e

cargo near build reproducible-wasm
mkdir -p ./res
cp ../target/near/sputnikdao_factory2/sputnikdao_factory2.wasm ./res/
