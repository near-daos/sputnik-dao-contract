#!/bin/bash
set -e

cargo near build reproducible-wasm
cp ../target/near/sputnikdao2/sputnikdao2.wasm ./res/
