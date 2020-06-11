#!/bin/bash -x
DFX=/home/sat/projects/sdk/target/x86_64-unknown-linux-musl/release/dfx

echo "Building the canister(s)"
cargo build --release

echo "Installing the canister(s)"
"$DFX" build && "$DFX" canister install --all

read -p "Press enter to add data buckets to the bigmap index"
"$DFX" canister call bigmap_index add_data_buckets "(vec { \"$($DFX canister id bigmap_data_0)\"; \"$($DFX canister id bigmap_data_1)\"; \"$($DFX canister id bigmap_data_2)\"; })"

read -p "Press enter to test get and put"

echo 'key "abc" is not yet in the data bucket:'
"$DFX" canister call bigmap_data_0 get '(vec { 97; 98; 99; })'

echo 'add key "abc" with value "def"'
"$DFX" canister call bigmap_data_0 put '(vec { 97; 98; 99; }, vec { 100; 101; 102; })'

echo 'key "abc" now in the data bucket'
"$DFX" canister call bigmap_data_0 get '(vec { 97; 98; 99; })'

echo "Test done!"
