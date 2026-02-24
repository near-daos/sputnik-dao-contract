#!/bin/bash
set -e

(cd sputnik-staking && sh build.sh)
(cd sputnikdao2 && sh build.sh)
(cd sputnikdao-factory2 && sh build.sh)
(cd test-token && sh build.sh)
