use crate::ic0;
use candid::Encode;
use data_encoding::BASE32_NOPAD;
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Waker};

pub mod context;
pub mod reflection;

use context::*;

#[derive(PartialEq, Clone, Eq)]
#[repr(transparent)]
pub struct CanisterId(pub Vec<u8>);

// TODO: move this to using the ic_agent canister.
impl CanisterId {
    pub fn from_str(s: &str) -> Result<Self, String> {
        let s_repl = s.to_ascii_uppercase().replace("-", "");

        let crc_and_can_id = BASE32_NOPAD
            .decode(s_repl.as_bytes())
            .map_err(|e| format!("Error while Base32 decoding {} => {}", s_repl, e))?;

        if crc_and_can_id.len() < 4 {
            return Err(format!(
                "Base32 decoded string is only {}  bytes long",
                crc_and_can_id.len()
            ));
        }
        let (digest_provided, can_id) = crc_and_can_id.split_at(4);

        let mut crc32hasher = crc32fast::Hasher::new();
        crc32hasher.update(can_id);
        let digest_calculated = crc32hasher.finalize().to_be_bytes();

        if digest_provided != digest_calculated {
            return Err("CRC32 checksum in the CanisterId is invalid".to_string());
        }

        Ok(CanisterId(Vec::from(can_id)))
    }
}

impl std::fmt::Display for CanisterId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let can_id_slice = &self.0;

        // Calculate CRC32 digest of the Canister ID
        let mut crc32hasher = crc32fast::Hasher::new();
        crc32hasher.update(can_id_slice);
        let crc32_bytes = crc32hasher.finalize().to_be_bytes();

        // Append the Canister ID bytes to the calculated CRC32 digest
        let mut crc_and_can_id = Vec::from(crc32_bytes);
        crc_and_can_id.extend(can_id_slice);

        // Base32-encode the concatenated bytes
        let s = BASE32_NOPAD.encode(&crc_and_can_id).to_ascii_lowercase();

        // Print with a separator - (dash) inserted every 5 characters.
        let mut s_peekable = s.chars().peekable();
        while s_peekable.peek().is_some() {
            let chunk: String = s_peekable.by_ref().take(5).collect();
            write!(f, "{}", chunk)?;
            if s_peekable.peek().is_some() {
                write!(f, "-")?;
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for CanisterId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CanisterId {}", self)
    }
}

impl From<Vec<u8>> for CanisterId {
    fn from(item: Vec<u8>) -> CanisterId {
        CanisterId { 0: item }
    }
}

impl Into<std::vec::Vec<u8>> for CanisterId {
    fn into(self) -> std::vec::Vec<u8> {
        self.0
    }
}

pub type CallResult<R> = Result<R, (RejectionCode, String)>;

/// Internal state for the Future when sending a call.
struct CallFutureState<R: serde::de::DeserializeOwned> {
    result: Option<CallResult<R>>,
    waker: Option<Waker>,
}
struct CallFuture<R: serde::de::DeserializeOwned> {
    // We basically use Rc instead of Arc (since we're single threaded), and use
    // RefCell instead of Mutex (because we cannot lock in WASM).
    state: Rc<RefCell<CallFutureState<R>>>,
}

impl<R: serde::de::DeserializeOwned> Future for CallFuture<R> {
    type Output = Result<R, (RejectionCode, String)>;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        let self_ref = Pin::into_ref(self);

        let mut state = self_ref.state.borrow_mut();

        if let Some(result) = state.result.take() {
            Poll::Ready(result)
        } else {
            state.waker = Some(context.waker().clone());
            Poll::Pending
        }
    }
}

/// Calls another canister and returns a future.
pub fn call<T: candid::CandidType, R: serde::de::DeserializeOwned>(
    id: CanisterId,
    method_name: &str,
    arg: Option<T>,
) -> impl Future<Output = CallResult<R>> {
    // The callback from IC dereferences the future from a raw pointer, assigns the
    // result and calls the waker. We cannot use a closure here because we pass raw
    // pointers to the System and back.
    fn callback<R: serde::de::DeserializeOwned>(state_ptr: *const RefCell<CallFutureState<R>>) {
        let state = unsafe { Rc::from_raw(state_ptr) };

        // Make sure to un-borrow_mut the state.
        {
            state.borrow_mut().result = Some(match reject_code() {
                RejectionCode::NoError => Ok(arg_data_1::<R>()),
                n => Err((n, reject_message())),
            });
        }

        if let Some(waker) = (|| state.borrow_mut().waker.take())() {
            // This is all to protect this little guy here which will call the poll() which
            // borrow_mut() the state as well. So we need to be careful to not double-borrow_mut.
            waker.wake()
        }
    };

    let data = match arg {
        None => candid::Encode!(),
        Some(data) => candid::Encode!(&data),
    }
    .expect("Could not encode arguments.");

    let callee = id.0;
    let state = Rc::new(RefCell::new(CallFutureState {
        result: None,
        waker: None,
    }));
    let state_ptr = Rc::into_raw(state.clone());

    let err_code = unsafe {
        ic0::call_simple(
            callee.as_ptr() as i32,
            callee.len() as i32,
            method_name.as_ptr() as i32,
            method_name.len() as i32,
            callback::<R> as i32,
            state_ptr as i32,
            callback::<R> as i32,
            state_ptr as i32,
            data.as_ptr() as i32,
            data.len() as i32,
        )
    };

    // 0 is a special error code, meaning call_simple call succeeded
    if err_code != 0 {
        let mut state = state.borrow_mut();
        state.result = Some(Err((
            RejectionCode::from(err_code),
            "Couldn't send message".to_string(),
        )));
    }

    CallFuture { state }
}

/// Calls another canister and returns a future.
pub fn call_no_return<T: candid::CandidType>(
    id: CanisterId,
    method_name: &str,
    arg: Option<T>,
) -> impl Future<Output = CallResult<()>> {
    // The callback from IC dereferences the future from a raw pointer, assigns the
    // result and calls the waker. We cannot use a closure here because we pass raw
    // pointers to the System and back.
    fn callback(state_ptr: *const RefCell<CallFutureState<()>>) {
        let state = unsafe { Rc::from_raw(state_ptr) };

        // Make sure to un-borrow_mut the state.
        {
            state.borrow_mut().result = Some(match reject_code() {
                RejectionCode::NoError => Ok(arg_data_0()),
                n => Err((n, reject_message())),
            });
        }

        if let Some(waker) = (|| state.borrow_mut().waker.take())() {
            // This is all to protect this little guy here which will call the poll() which
            // borrow_mut() the state as well. So we need to be careful to not double-borrow_mut.
            waker.wake()
        }
    };

    let data = match arg {
        None => candid::Encode!(),
        Some(data) => candid::Encode!(&data),
    }
    .expect("Could not encode arguments.");

    let callee = id.0;
    let state = Rc::new(RefCell::new(CallFutureState {
        result: None,
        waker: None,
    }));
    let state_ptr = Rc::into_raw(state.clone());

    let err_code = unsafe {
        ic0::call_simple(
            callee.as_ptr() as i32,
            callee.len() as i32,
            method_name.as_ptr() as i32,
            method_name.len() as i32,
            callback as i32,
            state_ptr as i32,
            callback as i32,
            state_ptr as i32,
            data.as_ptr() as i32,
            data.len() as i32,
        )
    };

    // 0 is a special error code, meaning call_simple call succeeded
    if err_code != 0 {
        let mut state = state.borrow_mut();
        state.result = Some(Err((
            RejectionCode::from(err_code),
            "Couldn't send message".to_string(),
        )));
    }

    CallFuture { state }
}

/// Prints the given message.
pub fn print<S: std::convert::AsRef<str>>(s: S) {
    let s = s.as_ref();
    unsafe {
        ic0::debug_print(s.as_ptr() as i32, s.len() as i32);
    }
}

/// Traps with the given message.
pub fn trap(message: &str) {
    unsafe {
        ic0::trap(message.as_ptr() as i32, message.len() as i32);
    }
}

#[cfg(test)]
mod tests;
