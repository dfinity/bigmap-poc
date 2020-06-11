use digest::generic_array::GenericArray;
use sha2::{Digest, Sha256};
pub mod data;
pub(crate) mod hashring;
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

#[cfg(not(target_arch = "wasm32"))]
mod lib_native;
#[cfg(not(target_arch = "wasm32"))]
pub use lib_native::*;

pub type Key = Vec<u8>;
pub type Val = Vec<u8>;
pub type Hashed = u64;

pub type Sha256Digest = GenericArray<u8, <Sha256 as Digest>::OutputSize>;

fn calc_sha256<T>(input: T) -> Sha256Digest
where
    T: std::convert::AsRef<[u8]>,
{
    let mut digest = Sha256::new();
    digest.update(input);
    digest.finalize()
}

/// Format and then print the formatted message
#[cfg(target_arch = "wasm32")]
#[macro_export]
macro_rules! println {
    ($fmt:expr) => (ic_cdk::print(format!($fmt)));
    ($fmt:expr, $($arg:tt)*) => (ic_cdk::print(format!($fmt, $($arg)*)));
}

#[cfg(not(target_arch = "wasm32"))]
#[macro_export]
macro_rules! println {
    ($fmt:expr) => (std::println!($fmt));
    ($fmt:expr, $($arg:tt)*) => (std::println!($fmt, $($arg)*));
}
