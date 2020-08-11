[![Rust](https://github.com/dfinity/bigmap-rs/workflows/Rust/badge.svg)](https://github.com/dfinity/bigmap-rs/actions)

# BigMap

BigMap is a simple, approachable library for building infinitely scalable key-value storage on the Internet Computer.

This is the Rust implementation of BigMap. There is also a Motoko-based implementation here: https://github.com/dfinity/motoko-bigmap

## Prerequisites

### IC SDK
To integrate BigMap with the Internet Computer applications, it's necessary to have the DFX version 0.6.1 or higher

```bash
DFX_VERSION=0.6.2 sh -ci "$(curl -fsSL https://sdk.dfinity.org/install.sh)"
```

Rust CDK is included with Big Map through `git subtree`, so it does not have to be separately downloaded and installed.

### Rust
Tested with Rust 1.43+. Please make sure you install the latest version.

#### If you don't have Rust already installed

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### If you do have Rust already installed, but may be an older version
```bash
rustup toolchain install stable
rustup override set stable
```

#### In both cases, add Wasm32 target in Rust
```bash
rustup target add wasm32-unknown-unknown
```


## Build and install canisters

```bash
git clone git@github.com:dfinity/big-map-rs.git
cd big-map-rs
cargo build --release
dfx build && dfx canister create --all && dfx canister install --all
dfx canister call bigmap add_data_buckets "(vec { \"$(dfx canister id bigmap_data_0)\"; \"$(dfx canister id bigmap_data_1)\"; \"$(dfx canister id bigmap_data_2)\"; })"
```

## Test

```bash
dfx canister call bigmap_data_0 get '(vec { 97; 98; 99; })'
# (null)
dfx canister call bigmap_data_0 put '(vec { 97; 98; 99; }, vec { 100; 101; 102; })'
# ()
dfx canister call bigmap_data_0 get '(vec { 97; 98; 99; })'
# (opt vec { 4; 5; 6; })
```

```bash
dfx canister call bigmap get '(vec { 97; 98; 99; })'
# (null)
dfx canister call bigmap put '(vec { 97; 98; 99; }, vec { 100; 101; 102; })'
# ()
dfx canister call bigmap get '(vec { 97; 98; 99; })'
# (opt vec { 4; 5; 6; })
```

