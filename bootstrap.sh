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
dfx canister create --all
dfx build

echo "Installing canisters"
dfx canister install bigmap --mode=reinstall

./bigmap-cli --set-data-bucket-wasm-binary target/wasm32-unknown-unknown/release/bigmap_data.wasm
./bigmap-cli --set-search-wasm-binary target/wasm32-unknown-unknown/release/bigmap_search.wasm

./bigmap-cli --put-and-fts-index bigsearch-works "BigSearch written in Rust works!"
