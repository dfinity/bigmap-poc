#!/bin/bash

set -e

#DFX=/home/sat/projects/sdk/target/x86_64-unknown-linux-musl/release/dfx
DFX=$(which dfx)

echo "Building the canister(s)"
cargo build --release --target wasm32-unknown-unknown

echo "Installing the canister(s)"
rm -rf canisters && "$DFX" build

"$DFX" canister create --all
"$DFX" canister install --all --mode=reinstall

# read -p "Press enter to add data buckets to the index"
"$DFX" canister call bigmap add_data_buckets "(vec { \"$($DFX canister id bigmap_data_0)\"; \"$($DFX canister id bigmap_data_1)\"; \"$($DFX canister id bigmap_data_2)\"; })"

read -p "Press enter to test get and put"

echo 'key "abc" through the index:'
"$DFX" canister call bigmap get '(vec { 97; 98; 99; })'
"$DFX" canister call bigmap put '(vec { 97; 98; 99; }, vec { 100; 101; 102; })'
"$DFX" canister call bigmap get '(vec { 97; 98; 99; })'

echo 'key "abc" directly with the data bucket:'
"$DFX" canister call bigmap_data_0 get '(vec { 97; 98; 99; })'

echo 'add key "abc" with value "def"'
"$DFX" canister call bigmap_data_0 put '(vec { 97; 98; 99; }, vec { 100; 101; 102; })'

echo 'key "abc" now in the data bucket'
"$DFX" canister call bigmap_data_0 get '(vec { 97; 98; 99; })'

echo "Test done!"
