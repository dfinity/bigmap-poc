#!/bin/bash

set -e

# Support bootstrapping from any folder
BASEDIR=$(cd "$(dirname "$0")"; pwd)
cd "$BASEDIR"

echo "Install dependencies"
npm install

echo "Building canisters"
# This command is ran from dfx build:
# cargo build --release --target wasm32-unknown-unknown
rm -rf canisters
dfx build

echo "Installing canisters"
dfx canister create bigmap
dfx canister install bigmap --mode=reinstall

./bigmap-cli --set-data-bucket-wasm-binary target/wasm32-unknown-unknown/release/bigmap_data.wasm
./bigmap-cli --maintenance
