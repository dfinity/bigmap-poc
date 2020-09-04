use digest::generic_array::GenericArray;
use sha2::{Digest, Sha256};
pub mod data;
pub(crate) mod hashring;
#[allow(dead_code)]
pub(crate) mod hashring_sha256;
pub mod index;
use candid::CandidType;

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

use candid::Encode;

#[derive(CandidType, serde::Deserialize)]
pub struct EmptyBlob {}

impl EmptyBlob {
    pub fn encode() -> Vec<u8> {
        Encode!().unwrap()
    }
}

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

#[derive(CandidType, serde::Deserialize, Debug)]
struct CanisterIdRecord {
    canister_id: candid::Principal,
}

#[allow(dead_code)]
pub async fn create_new_canister() -> Result<CanisterId, String> {
    let management_canister = ic_cdk::CanisterId::from(Vec::new());
    let new_can_id_record: CanisterIdRecord =
        match ic_cdk::call(management_canister, "create_canister", Some(())).await {
            Ok(res) => res,
            Err(err) => {
                ic_cdk::println!("Error invoking create_canister: {:?} {}", err.0, err.1);
                return Err(err.1);
            }
        };

    let new_can_id = CanisterId::from(new_can_id_record.canister_id.as_slice());
    Ok(new_can_id)
}

use serde::{Deserialize, Serialize};
/// The mode with which a canister is installed.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Eq, Hash, CandidType)]
pub enum CanisterInstallMode {
    /// A fresh install of a new canister.
    #[serde(rename = "install")]
    Install,
    /// Reinstalling a canister that was already installed.
    #[serde(rename = "reinstall")]
    Reinstall,
    /// Upgrade an existing canister.
    #[serde(rename = "upgrade")]
    Upgrade,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct InstallCodeArgs {
    pub mode: CanisterInstallMode,
    pub canister_id: candid::Principal,
    pub wasm_module: Vec<u8>,
    pub arg: Vec<u8>,
    pub compute_allocation: Option<u64>,
    pub memory_allocation: Option<u64>,
}
use std::convert::TryFrom;

impl InstallCodeArgs {
    pub fn new(
        mode: CanisterInstallMode,
        canister_id: CanisterId,
        wasm_module: Vec<u8>,
        arg: Vec<u8>,
        compute_allocation: Option<u64>,
        memory_allocation: Option<u64>,
    ) -> Self {
        Self {
            mode,
            canister_id: candid::Principal::try_from(canister_id.0)
                .expect("Failed to make principal from canister_id"),
            wasm_module,
            arg,
            compute_allocation,
            memory_allocation,
        }
    }

    pub fn get_canister_id(&self) -> CanisterId {
        CanisterId::from(self.canister_id.as_slice())
    }

    pub fn encode(&self) -> Vec<u8> {
        Encode!(&self).unwrap()
    }
}

pub async fn install_canister_code(
    canister_id: CanisterId,
    wasm_module: Vec<u8>,
) -> Result<(), String> {
    if wasm_module.is_empty() {
        return Err("Empty wasm module provided for canister installation".to_string());
    }

    let management_canister = ic_cdk::CanisterId::from(Vec::new());

    let install_code_args = InstallCodeArgs {
        mode: CanisterInstallMode::Install,
        canister_id: candid::Principal::try_from(canister_id.0)
            .expect("Failed to make principal from canister_id"),
        wasm_module,
        arg: Vec::new(),
        compute_allocation: None,
        memory_allocation: None,
    };

    match ic_cdk::call_no_return(management_canister, "install_code", Some(install_code_args)).await
    {
        Ok(res) => res,
        Err(err) => {
            ic_cdk::println!("Error invoking install_code: {:?} {}", err.0, err.1);
            return Err(err.1);
        }
    };

    Ok(())
}
