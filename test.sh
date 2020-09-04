#!/bin/bash

set -e

echo "Building the canister(s)"
cargo build --release --target wasm32-unknown-unknown

echo "Installing the canister(s)"
rm -rf canisters
dfx build

dfx canister create bigmap
dfx canister install bigmap --mode=reinstall

./bigmap-cli --set-data-bucket-wasm-binary target/wasm32-unknown-unknown/release/bigmap_data.wasm
./bigmap-cli --maintenance

echo 'Get key "abc" before adding it'
./bigmap-cli --get abc
echo 'Insert key "abc" ==> "def"'
./bigmap-cli --put abc def
echo 'Get key "abc" after adding it'
./bigmap-cli --get abc

echo "Test done!"
