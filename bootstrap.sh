#!/bin/bash

set -e

# Support bootstrapping from any folder
BASEDIR=$(cd "$(dirname "$0")"; pwd)
cd "$BASEDIR"

if [[ "$1" == "sodium" ]]; then
  echo "Bootstraping on the Sodium network"
  NETWORK="--network sodium"
  if [[ -z "$DFX_CREDS_USER" || -z "$DFX_CREDS_PASS" || -z "$DFX_NETWORK" ]]; then
    echo "Please make sure you set the following environment variables:"
    echo "export DFX_CREDS_USER={username}"
    echo "export DFX_CREDS_PASS={password}"
    echo "export DFX_NETWORK={network}"
    exit 1
  fi
else
  if [[ "$1" == "sodium-test" ]]; then
    echo "Bootstraping on the Sodium-test network"
    NETWORK="--network sodium-test"
    if [[ -z "$DFX_CREDS_USER" || -z "$DFX_CREDS_PASS" || -z "$DFX_NETWORK" ]]; then
      echo "Please make sure you set the following environment variables:"
      echo "export DFX_CREDS_USER={username}"
      echo "export DFX_CREDS_PASS={password}"
      echo "export DFX_NETWORK={network}"
      exit 1
    fi
  else
    echo "Bootstraping on the local instance"
  fi
fi

echo "Install dependencies"
npm install

echo "Building canisters"
# This command is ran from dfx build:
# cargo build --release --target wasm32-unknown-unknown
rm -rf canisters
dfx canister $NETWORK create --all
dfx build $NETWORK

echo "Installing canisters"
dfx canister $NETWORK install bigmap --mode=reinstall

./bigmap-cli --set-data-bucket-wasm-binary target/wasm32-unknown-unknown/release/bigmap_data.wasm
./bigmap-cli --set-search-wasm-binary target/wasm32-unknown-unknown/release/bigmap_search.wasm

./bigmap-cli --put-and-fts-index bigsearch-works "BigSearch written in Rust works!"
