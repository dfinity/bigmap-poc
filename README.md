[![Rust](https://github.com/dfinity/bigmap-rs/workflows/Rust/badge.svg)](https://github.com/dfinity/bigmap-rs/actions)

# BigMap

BigMap is a simple, approachable library for building infinitely scalable key-value storage on the Internet Computer.

This is the Rust implementation of BigMap. There is also a Motoko-based implementation here: https://github.com/dfinity/motoko-bigmap

## Prerequisites

### Dependencies

* Rust compiler 1.45+ (and cargo)
* cmake (optional but recommended)
* IC SDK (DFX) 0.6.9+
* Nodejs v12

### IC SDK
To integrate BigMap with the Internet Computer applications, it's necessary to have the DFX version 0.6.9 or higher

```bash
DFX_VERSION=0.6.9 sh -ci "$(curl -fsSL https://sdk.dfinity.org/install.sh)"
```

Rust CDK is included with Big Map through `git subtree`, so it does not have to be separately downloaded and installed.

### Rust
Tested with Rust 1.45+. Please make sure you install the latest version.

#### Make sure you're running the latest version of Rust, with wasm32 target

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
rustup toolchain install stable
rustup override set stable
rustup target add wasm32-unknown-unknown
```

## Mac specific

On MacOS Catalina it's necessary to perform particular steps in order to install nodejs.
You may need to follow these steps if you get the following error:
```
gyp: No Xcode or CLT version detected!
```

A solution which seems to work for many people is to run the following command in a terminal:
```
xcode-select --install
```

If that doesn't work, check the complete instructions at
https://github.com/nodejs/node-gyp/blob/master/macOS_Catalina.md

Additional discussions and instructions are available at:
https://github.com/nodejs/node-gyp/issues/1927#issuecomment-544499926
and
https://github.com/nodejs/node-gyp/issues/1927#issuecomment-544507444

## DFX replica start

FIXME

## Build and install canisters

Simply run `./bootstrap.sh`, or 

```bash
git clone git@github.com:dfinity/big-map-rs.git
cd big-map-rs

npm install
dfx canister create --all
dfx build
dfx canister install bigmap --mode=reinstall
./bigmap-cli --set-data-bucket-wasm-binary target/wasm32-unknown-unknown/release/bigmap_data.wasm
./bigmap-cli --set-search-wasm-binary target/wasm32-unknown-unknown/release/bigmap_search.wasm
```

## Test

You can either take a look at `test.sh` for a complete set of test steps, or you can selectively run the below commands:

```bash
dfx canister call bigmap get '(vec { 97; 98; 99; })'
# (null)
dfx canister call bigmap put '(vec { 97; 98; 99; }, vec { 100; 101; 102; })'
# ()
dfx canister call bigmap get '(vec { 97; 98; 99; })'
# (opt vec { 4; 5; 6; })
```

It is also possible to talk directly to the data bucket canisters, but this is likely only useful during development or debugging.
In this case it's necessary to know the CanisterId, which is printed in the replica debug logs during creation.

For example:
```bash
dfx canister call tup4c-ks6aa-aaaaa-aaaaa-aaaaa-aaaaa-aaaaa-q get '(vec { 97; 98; 99; })'
# (null)
dfx canister call tup4c-ks6aa-aaaaa-aaaaa-aaaaa-aaaaa-aaaaa-q put '(vec { 97; 98; 99; }, vec { 100; 101; 102; })'
# ()
dfx canister call tup4c-ks6aa-aaaaa-aaaaa-aaaaa-aaaaa-aaaaa-q get '(vec { 97; 98; 99; })'
# (opt vec { 4; 5; 6; })
```
