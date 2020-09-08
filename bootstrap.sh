#!/bin/bash

set -e

dfx build
dfx canister install bigmap --mode=reinstall
./bigmap-cli --set-data-bucket-wasm-binary target/wasm32-unknown-unknown/release/bigmap_data.wasm
./bigmap-cli --maintenance
