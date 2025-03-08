#!/bin/bash
set -e

cargo +stable build --target wasm32-unknown-unknown --release
cp target/wasm32-unknown-unknown/release/sputnik_staking.wasm ./sputnik-staking/res/
cp target/wasm32-unknown-unknown/release/sputnikdao2.wasm ./sputnikdao2/res/
cp target/wasm32-unknown-unknown/release/sputnikdao_factory2.wasm ./sputnikdao-factory2/res/
cp target/wasm32-unknown-unknown/release/test_token.wasm ./test-token/res/