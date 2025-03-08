#!/bin/bash
set -e

(cd sputnik-staking && cargo near build non-reproducible-wasm --no-abi)
(cd sputnikdao2 && cargo near build non-reproducible-wasm --no-abi)
(cd sputnikdao-factory2 && cargo near build non-reproducible-wasm --no-abi)
(cd test-token && cargo near build non-reproducible-wasm --no-abi)
cp target/near/sputnik_staking/sputnik_staking.wasm ./sputnik-staking/res/
cp target/near/sputnikdao2/sputnikdao2.wasm ./sputnikdao2/res/
cp target/near/sputnikdao_factory2/sputnikdao_factory2.wasm ./sputnikdao-factory2/res/
cp target/near/test_token/test_token.wasm ./test-token/res/