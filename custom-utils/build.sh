#!/bin/bash
set -e

RUSTFLAGS='-C link-arg=-s' cargo +stable build --target wasm32-unknown-unknown --release
cp ../target/wasm32-unknown-unknown/release/custom_utils.wasm ./res/
