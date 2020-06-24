![Rust](https://github.com/dfinity/bigmap-rs/workflows/Rust/badge.svg)

# BigMap

BigMap is a simple, approachable library for building infinitely scalable key-value storage on the Internet Computer.

This is the Rust implementation of BigMap. There is also a Motoko-based implementation here: https://github.com/dfinity/motoko-bigmap

## Prerequisites

Rust CDK is included with Big Map through `git subtree`, so it does not have to be separately downloaded and installed. You do have to clone this repo, though :)

```bash
git clone git@github.com:dfinity/big-map-rs.git
cd big-map-rs
```


To integrate BigMap with the Internet Computer applications, it's necessary to have the DFX version 0.5.8 or higher

```bash
DFX_VERSION=0.5.8 sh -ci "$(curl -fsSL https://sdk.dfinity.org/install.sh)"
```

<!--
```bash
cd sdk
cargo build --release
alias dfx=$(realpath target/x86_64-unknown-linux-musl/release/dfx)
cd ../big-map-rs
```
-->

Tested with Rust 1.43.0
```bash
rustc --version
rustc 1.43.0 (4fb7144ed 2020-04-20)
```

## Build and install canisters

```bash
cargo build --release
dfx build && dfx canister install --all
dfx canister call bigmap_index add_data_buckets "(vec { \"$(dfx canister id bigmap_data_0)\"; \"$(dfx canister id bigmap_data_1)\"; \"$(dfx canister id bigmap_data_2)\"; })"
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
dfx canister call bigmap_index get '(vec { 97; 98; 99; })'
# (null)
dfx canister call bigmap_index put '(vec { 97; 98; 99; }, vec { 100; 101; 102; })'
# ()
dfx canister call bigmap_index get '(vec { 97; 98; 99; })'
# (opt vec { 4; 5; 6; })
```


## Super Powers realized by BigMap

The BigMap library touches on the following aspects of the vision of the Internet Computer's "super powers":

### 3. "Data Without Storage"

This library offers building blocks for application-specific, in-memory data abstractions that scale.  This goal relates to the following excerpt:

> "Imagine that whenever you...created a collection...no matter how big...those abstractions would be sufficient to ensure the persistence of the data, without the need to marshal it out to databases or in and out of files for safe keeping" (text from [10 Super Powers List]).

### 6. "Treat systems like software libraries"

This library can be integrated as a library.  This library can also be integrated as a service.

> "Third party systems and services providing powerful functionality...can be easily integrated into your own system just like you integrate software libraries today." (text from [10 Super Powers List]).


### 9. "Scale out"

As mentioned above, this library will permit unlimited scaling.  Under the hood, this is achieved via _cross-subnetwork_ application-level abstractions, instantiated from generic abstractions provided by BigMap.

> "Build mass market internet services using the canister model (each canister can only grow to the capacity of a single subnetwork, but you can build a system from any number of canisters)" (text from [10 Super Powers List]).


### References

- [10 Super Powers List](https://docs.google.com/document/d/1Bxnn0--YoB_2sVWm33jWXhDFxsyOEhYG0KU7G1SL_q8/edit)
