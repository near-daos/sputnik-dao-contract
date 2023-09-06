#!/bin/bash
set -e

RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release
cd ..
mkdir -p res
cp target/wasm32-unknown-unknown/release/test_token.wasm res/
