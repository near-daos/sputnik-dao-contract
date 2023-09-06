#!/bin/bash
set -e

cargo build --target wasm32-unknown-unknown --release
mkdir -p res
cp target/wasm32-unknown-unknown/release/*.wasm ./res/
