use crate::CanisterId;
use candid::CandidType;

#[derive(CandidType, serde::Deserialize, Debug)]
struct CanisterIdRecord {
    canister_id: candid::Principal,
}

pub async fn subnet_create_new_canister() -> Result<CanisterId, String> {
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

pub async fn subnet_install_canister_code(
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

pub async fn subnet_raw_rand() -> Result<Vec<u8>, String> {
    let management_canister = ic_cdk::CanisterId::from(Vec::new());
    let rnd_buffer: Vec<u8> = match ic_cdk::call(management_canister, "raw_rand", Some(())).await {
        Ok(result) => result,
        Err(err) => {
            ic_cdk::println!("Error invoking raw_rand: {:?} {}", err.0, err.1);
            return Err(err.1);
        }
    };

    Ok(rnd_buffer.to_vec())
}
