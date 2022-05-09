#!/bin/bash
set -e

# build the things
./build.sh

export NEAR_ENV=mainnet
export FACTORY=near

if [ -z ${NEAR_ACCT+x} ]; then
  export NEAR_ACCT=sputnik-dao.$FACTORY
else
  export NEAR_ACCT=$NEAR_ACCT
fi

export FACTORY_ACCOUNT_ID=sputnik-dao.$FACTORY
export MAX_GAS=300000000000000

# Loop All accounts and deploy v2 gas fixes
# NOTE: If this fails, the full access key could be wrong.
# NOTE: If this fails, the account might not have enough funds for new code storage.
accounts=("collabs" "wiki" "openshards" "codame" "pixeltoken" "curators" "multicall" "marmaj" "hype" "mochi" "news" "famjam" "genesis" "hak" "peter" "nearweek" "thekindao" "shrm" "skyward" "metapool" "prod_dev" "pulse" "stardust" "city-nodes" "audit" "simplegames" "auctionhouse" "wyosky" "grain-lang" "jascha" "swarming" "roketo" "rarity" "information" "millionaire-raccoons-dao" "aurora" "mindful" "maximum-viable-potential" "raritydao" "pulse-markets" "nearprotocoltamil" "terrans" "now-fund-this" "rucommunity" "cpgtest" "nyc-sports" "learnnear" "lisboa-node" "jinn" "transform" "art" "nearsighted" "kindessgrocerycoop" "devco" "hfq" "catalygraphy" "nftbuzz" "yaway" "localmakermart" "nihilism_fulltime" "flymoon" "blackvirtualmap" "abra" "nft")
for (( e=0; e<=${#accounts[@]} - 1; e++ ))
do
  DAO_ACCOUNT_ID="${accounts[e]}.${FACTORY_ACCOUNT_ID}"
  echo "Upgrading: ${DAO_ACCOUNT_ID}"

  # FOR DEPLOYING v2 with the gas fix
  # near deploy --wasmFile sputnikdao2-gasfix/res/sputnikdao2_gasfix.wasm --accountId $DAO_ACCOUNT_ID --initGas $MAX_GAS --force
  # # FOR DEPLOYING v3 directly (SHOULD BE AVOIDED IF POSSIBLE!)
  # near deploy --wasmFile sputnikdao2/res/sputnikdao2.wasm --accountId $FACTORY_ACCOUNT_ID --initGas $MAX_GAS --force

  echo "Deployed ${DAO_ACCOUNT_ID}: Go to https://explorer.near.org/accounts/${DAO_ACCOUNT_ID} and check the code_hash"
done
