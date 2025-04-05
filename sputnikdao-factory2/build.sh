#!/bin/bash
set -e

cargo near build reproducible-wasm
cp ../target/near/sputnikdao_factory2/sputnikdao_factory2.wasm ./res/
