#!/bin/bash
set -e

RUSTFLAGS='-C link-arg=-s' cargo +stable build --target wasm32-unknown-unknown --release
wasm-opt --converge -Oz --signext-lowering ../target/wasm32-unknown-unknown/release/sputnikdao_factory2.wasm -o ./res/sputnikdao_factory2.wasm
