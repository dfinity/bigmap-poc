#!/bin/bash

set -e

echo "Building canisters"
cargo build --release --target wasm32-unknown-unknown
rm -rf canisters
dfx build

echo "Installing canisters"
dfx canister create bigmap
dfx canister install bigmap --mode=reinstall

./bigmap-cli --set-data-bucket-wasm-binary target/wasm32-unknown-unknown/release/bigmap_data.wasm
./bigmap-cli --maintenance
