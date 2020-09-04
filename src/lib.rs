use digest::generic_array::GenericArray;
use sha2::{Digest, Sha256};
pub mod data;
pub(crate) mod hashring;
#[allow(dead_code)]
pub(crate) mod hashring_sha256;
pub mod index;

/********************************************************************
     ____  _         __  __
    + __ )(_) __ _  +  \/  + __ _ _ __
    |  _ \+ +/ _` + | +\/+ |/ _` + '_ \
    | +_) | | (_+ | | |  | | (_+ | +_) +
    +____/+_+\__, + +_+  +_+\__,_| .__/
            +___/               +_+
                        Application architecture
                                           +--------------+
        +---------------+                  | BigMap Data  |
        | BigMap Index  |                  | Bucket Can.  |
        | Canister      |         +------->---------------+
        +-------+-------+         |        +--------------+
                ^                 |        | BigMap Data  |
                |                 +------->+ Bucket Can.  |
                |                 |        +--------------+
        +-------+-------+         |        +--------------+
        | BigMap client +---------+--------+ BigMap Data  |
        | App Canister  |                  | Bucket Can.  |
        +---------------+                  +--------------+
                                                    .
                                                    .
                                                    .

********************************************************************/

#[cfg(target_arch = "wasm32")]
mod lib_wasm32;
#[cfg(target_arch = "wasm32")]
pub use lib_wasm32::*;

mod canister_management;
pub use canister_management::{create_new_canister, install_canister_code};

#[cfg(not(target_arch = "wasm32"))]
mod lib_native;
#[cfg(not(target_arch = "wasm32"))]
pub use lib_native::*;

pub type Key = Vec<u8>;
pub type Val = Vec<u8>;
pub type Sha2Vec = Vec<u8>;

pub type Sha256Digest = GenericArray<u8, <Sha256 as Digest>::OutputSize>;

fn calc_sha256<T>(input: T) -> Sha256Digest
where
    T: std::convert::AsRef<[u8]>,
{
    let mut digest = Sha256::new();
    digest.update(input);
    digest.finalize()
}

fn sha256_digest_from_vec(input: &Vec<u8>) -> Sha256Digest {
    let mut input = input.clone();
    input.resize(32, 0); // ensure proper size
    *Sha256Digest::from_slice(&input)
}

// #[allow(dead_code)]
// pub mod dfn_candid;
// #[allow(dead_code)]
// mod dfn_core;
// #[allow(dead_code)]
// mod dfn_futures;
// // pub use dfn_candid;
// pub use dfn_candid::call_candid;
