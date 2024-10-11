#!/bin/bash
set -e

const NEAR_SANDBOX_URL: &str = "https://s3-us-west-1.amazonaws.com/build.nearprotocol.com/nearcore/Linux-x86_64/master/e7ff91329e9a7cb6e38b6409dfa2d0bc9c058f6f/near-sandbox.tar.gz";

RUSTFLAGS='-C link-arg=-s' cargo +stable build --target wasm32-unknown-unknown --release
cp ../target/wasm32-unknown-unknown/release/sputnikdao_factory2.wasm ./res/