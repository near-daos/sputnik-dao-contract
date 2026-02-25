#!/bin/bash
set -e

(cd sputnik-staking && cargo near build reproducible-wasm)
(cd sputnikdao2 && cargo near build reproducible-wasm)
(cd sputnikdao-factory2 && cargo near build reproducible-wasm)
(cd test-token && cargo near build reproducible-wasm)
