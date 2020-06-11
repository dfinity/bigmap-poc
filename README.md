# Prerequisites

Have Rust SDK/CDK and the Big Map in the same directory:

It's necessary to have the DFX version >= 0.5.7-28

```bash
git clone git@github.com:dfinity-lab/sdk.git
git clone git@github.com:dfinity-lab/rust-cdk.git
git clone git@github.com:dfinity/big-map-rs.git
```


```bash
cd sdk
cargo build --release
alias dfx=$(realpath target/x86_64-unknown-linux-musl/release/dfx)
cd ../big-map-rs
```

Tested with Rust 1.43.0
```bash
rustc --version
rustc 1.43.0 (4fb7144ed 2020-04-20)
```

# Build and install canisters

```bash
cargo build --release
dfx build && dfx canister install --all
```

# Test

```bash
dfx canister call data_0 get '(vec { 1; 2; 3; })'
# (null)
dfx canister call data_0 put '(vec { 1; 2; 3; }, vec { 4; 5; 6; })'
# ()
dfx canister call data_0 get '(vec { 1; 2; 3; })'
# (opt vec { 4; 5; 6; })
```
