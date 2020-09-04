use crate::{ic0, reflection};
use candid::{Decode, Encode, IDLArgs};

/// Rejection code from calling another canister.
/// These can be obtained either using `reject_code()` or `reject_result()`.
#[repr(i32)]
#[derive(Debug)]
pub enum RejectionCode {
    NoError = 0,

    SysFatal = 1,
    SysTransient = 2,
    DestinationInvalid = 3,
    CanisterReject = 4,
    CanisterError = 5,

    Unknown,
}

impl From<i32> for RejectionCode {
    fn from(code: i32) -> Self {
        match code {
            0 => RejectionCode::NoError,
            1 => RejectionCode::SysFatal,
            2 => RejectionCode::SysTransient,
            3 => RejectionCode::DestinationInvalid,
            4 => RejectionCode::CanisterReject,
            5 => RejectionCode::CanisterError,
            _ => RejectionCode::Unknown,
        }
    }
}

/// Returns a result that maps over the call; it will be Ok(T) if
/// the call succeeded (with T being the arg_data), and [reject_message()] if it failed.
pub fn result<T: serde::de::DeserializeOwned>() -> Result<T, String> {
    match reject_code() {
        RejectionCode::NoError => Ok(arg_data_1::<T>()),
        _ => Err(reject_message()),
    }
}

/// Get the rejection code for the call.
pub fn reject_code() -> RejectionCode {
    let code = unsafe { ic0::msg_reject_code() };
    RejectionCode::from(code)
}

/// Returns the rejection message.
pub fn reject_message() -> String {
    let len: u32 = unsafe { ic0::msg_reject_msg_size() as u32 };
    let mut bytes = vec![0; len as usize];
    unsafe {
        ic0::msg_reject_msg_copy(bytes.as_mut_ptr() as i32, 0, len as i32);
    }
    String::from_utf8_lossy(&bytes).to_string()
}

/// Reject the current call with the message.
pub fn reject(message: &str) {
    let err_message = message.as_bytes();
    unsafe {
        ic0::msg_reject(err_message.as_ptr() as i32, err_message.len() as i32);
    }
}

/// Get the sender principal ID.
pub fn sender() -> Vec<u8> {
    reflection::caller()
}

pub(crate) unsafe fn reply_raw(reply: &[u8]) {
    ic0::msg_reply_data_append(reply.as_ptr() as i32, reply.len() as i32);
    ic0::msg_reply();
}

pub fn reply<T: candid::CandidType>(reply: T) {
    let bytes = Encode!(&reply).expect("Could not encode reply.");
    unsafe {
        reply_raw(&bytes);
    }
}

pub fn reply_empty() {
    let bytes = Encode!().expect("Could not encode reply.");
    unsafe {
        reply_raw(&bytes);
    }
}

pub(crate) unsafe fn arg_data_raw() -> Vec<u8> {
    let len: usize = ic0::msg_arg_data_size() as usize;
    let mut bytes = vec![0u8; len as usize];
    ic0::msg_arg_data_copy(bytes.as_mut_ptr() as i32, 0, len as i32);
    bytes
}

pub fn arg_data_is_empty() -> bool {
    unsafe { ic0::msg_arg_data_size() == 0 }
}

/// Decodes 0 argument from the arguments data.
pub fn arg_data_0() {
    unsafe {
        let raw_data = arg_data_raw();
        match Decode!(&raw_data) {
            Ok(res) => res,
            Err(err) => {
                let err_msg = format!(
                    "Failed to deserialize {} = {} to 0 arguments; Decode error: {}",
                    hex::encode(&raw_data),
                    IDLArgs::from_bytes(&raw_data).unwrap(),
                    err.to_string()
                );
                panic!(err_msg);
            }
        }
    }
}

/// Decodes 1 argument from the arguments data.
pub fn arg_data_1<A>() -> A
where
    A: serde::de::DeserializeOwned,
{
    unsafe {
        let raw_data = arg_data_raw();
        match Decode!(&raw_data, A) {
            Ok(res) => res,
            Err(err) => {
                let err_msg = format!(
                    "Failed to deserialize {} = {} to 1 argument; Decode error: {}",
                    hex::encode(&raw_data),
                    IDLArgs::from_bytes(&raw_data).unwrap(),
                    err.to_string()
                );
                panic!(err_msg);
            }
        }
    }
}

/// Decodes 2 argument from the arguments data.
pub fn arg_data_2<A, B>() -> (A, B)
where
    A: serde::de::DeserializeOwned,
    B: serde::de::DeserializeOwned,
{
    unsafe {
        let raw_data = arg_data_raw();
        match Decode!(&raw_data, A, B) {
            Ok(res) => res,
            Err(err) => {
                let err_msg = format!(
                    "Failed to deserialize {} = {} to 2 argument; Decode error: {}",
                    hex::encode(&raw_data),
                    IDLArgs::from_bytes(&raw_data).unwrap(),
                    err.to_string()
                );
                panic!(err_msg);
            }
        }
    }
}

/// Decodes 3 argument from the arguments data.
pub fn arg_data_3<A, B, C>() -> (A, B, C)
where
    A: serde::de::DeserializeOwned,
    B: serde::de::DeserializeOwned,
    C: serde::de::DeserializeOwned,
{
    unsafe {
        let raw_data = arg_data_raw();
        match Decode!(&raw_data, A, B, C) {
            Ok(res) => res,
            Err(err) => {
                let err_msg = format!(
                    "Failed to deserialize {} = {} to 3 arguments; Decode error: {}",
                    hex::encode(&raw_data),
                    IDLArgs::from_bytes(&raw_data).unwrap(),
                    err.to_string()
                );
                panic!(err_msg);
            }
        }
    }
}

/// Decodes 4 argument from the arguments data.
pub fn arg_data_4<A, B, C, D>() -> (A, B, C, D)
where
    A: serde::de::DeserializeOwned,
    B: serde::de::DeserializeOwned,
    C: serde::de::DeserializeOwned,
    D: serde::de::DeserializeOwned,
{
    unsafe {
        let raw_data = arg_data_raw();
        match Decode!(&raw_data, A, B, C, D) {
            Ok(res) => res,
            Err(err) => {
                let err_msg = format!(
                    "Failed to deserialize {} = {} to 4 arguments; Decode error: {}",
                    hex::encode(&raw_data),
                    IDLArgs::from_bytes(&raw_data).unwrap(),
                    err.to_string()
                );
                panic!(err_msg);
            }
        }
    }
}

/// Decodes 5 argument from the arguments data.
pub fn arg_data_5<A, B, C, D, E>() -> (A, B, C, D, E)
where
    A: serde::de::DeserializeOwned,
    B: serde::de::DeserializeOwned,
    C: serde::de::DeserializeOwned,
    D: serde::de::DeserializeOwned,
    E: serde::de::DeserializeOwned,
{
    unsafe {
        let raw_data = arg_data_raw();
        match Decode!(&raw_data, A, B, C, D, E) {
            Ok(res) => res,
            Err(err) => {
                let err_msg = format!(
                    "Failed to deserialize {} = {} to 5 arguments; Decode error: {}",
                    hex::encode(&raw_data),
                    IDLArgs::from_bytes(&raw_data).unwrap(),
                    err.to_string()
                );
                panic!(err_msg);
            }
        }
    }
}

/// Decodes 6 argument from the arguments data.
pub fn arg_data_6<A, B, C, D, E, F>() -> (A, B, C, D, E, F)
where
    A: serde::de::DeserializeOwned,
    B: serde::de::DeserializeOwned,
    C: serde::de::DeserializeOwned,
    D: serde::de::DeserializeOwned,
    E: serde::de::DeserializeOwned,
    F: serde::de::DeserializeOwned,
{
    unsafe {
        let raw_data = arg_data_raw();
        match Decode!(&raw_data, A, B, C, D, E, F) {
            Ok(res) => res,
            Err(err) => {
                let err_msg = format!(
                    "Failed to deserialize {} = {} to 6 arguments; Decode error: {}",
                    hex::encode(&raw_data),
                    IDLArgs::from_bytes(&raw_data).unwrap(),
                    err.to_string()
                );
                panic!(err_msg);
            }
        }
    }
}

/// Decodes 7 argument from the arguments data.
pub fn arg_data_7<A, B, C, D, E, F, G>() -> (A, B, C, D, E, F, G)
where
    A: serde::de::DeserializeOwned,
    B: serde::de::DeserializeOwned,
    C: serde::de::DeserializeOwned,
    D: serde::de::DeserializeOwned,
    E: serde::de::DeserializeOwned,
    F: serde::de::DeserializeOwned,
    G: serde::de::DeserializeOwned,
{
    unsafe {
        let raw_data = arg_data_raw();
        match Decode!(&raw_data, A, B, C, D, E, F, G) {
            Ok(res) => res,
            Err(err) => {
                let err_msg = format!(
                    "Failed to deserialize {} = {} to 7 arguments; Decode error: {}",
                    hex::encode(&raw_data),
                    IDLArgs::from_bytes(&raw_data).unwrap(),
                    err.to_string()
                );
                panic!(err_msg);
            }
        }
    }
}

/// Decodes 8 argument from the arguments data.
pub fn arg_data_8<A, B, C, D, E, F, G, H>() -> (A, B, C, D, E, F, G, H)
where
    A: serde::de::DeserializeOwned,
    B: serde::de::DeserializeOwned,
    C: serde::de::DeserializeOwned,
    D: serde::de::DeserializeOwned,
    E: serde::de::DeserializeOwned,
    F: serde::de::DeserializeOwned,
    G: serde::de::DeserializeOwned,
    H: serde::de::DeserializeOwned,
{
    unsafe {
        let raw_data = arg_data_raw();
        match Decode!(&raw_data, A, B, C, D, E, F, G, H) {
            Ok(res) => res,
            Err(err) => {
                let err_msg = format!(
                    "Failed to deserialize {} = {} to 8 arguments; Decode error: {}",
                    hex::encode(&raw_data),
                    IDLArgs::from_bytes(&raw_data).unwrap(),
                    err.to_string()
                );
                panic!(err_msg);
            }
        }
    }
}
