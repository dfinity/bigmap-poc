// Load the allocator
cfg_if::cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        use wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc<'_> = wee_alloc::WeeAlloc::INIT;
    }
}

// use futures::{CallFuture, RefCounted};
use std::{cell::RefCell, future::Future};

#[derive(PartialEq, Clone, Eq)]
#[repr(transparent)]
pub struct CanisterId(pub Vec<u8>);

#[allow(dead_code)]
#[cfg(target_arch = "wasm32")]
mod ic0 {
    #[link(wasm_import_module = "ic0")]
    extern "C" {
        pub fn canister_self_copy(dst: u32, offset: u32, size: u32) -> ();
        pub fn canister_self_size() -> u32;
        pub fn debug_print(offset: u32, size: u32) -> ();
        pub fn msg_arg_data_copy(dst: u32, offset: u32, size: u32) -> ();
        pub fn msg_arg_data_size() -> u32;
        pub fn msg_caller_copy(dst: u32, offset: u32, size: u32) -> ();
        pub fn msg_caller_size() -> u32;
        pub fn msg_reject(src: u32, size: u32) -> ();
        pub fn msg_reject_code() -> i32;
        pub fn msg_reject_msg_copy(dst: u32, offset: u32, size: u32) -> ();
        pub fn msg_reject_msg_size() -> u32;
        pub fn msg_reply() -> ();
        pub fn msg_reply_data_append(offset: u32, size: u32) -> ();
        pub fn trap(offset: u32, size: u32) -> ();
        pub fn call_simple(
            callee_src: u32,
            callee_size: u32,
            name_src: u32,
            name_size: u32,
            reply_fun: usize,
            reply_env: u32,
            reject_fun: usize,
            reject_env: u32,
            data_src: u32,
            data_size: u32,
        ) -> i32;
        pub fn stable_size() -> u32;
        pub fn ic0_stable_grow(additional_pages: u32) -> i32;
        pub fn ic0_stable_read(dst: u32, offset: u32, size: u32) -> ();
        pub fn ic0_stable_write(offset: u32, src: u32, size: u32) -> ();
    }
}

/*
These stubs exist for when you're compiling this code not on a canister. If you
delete this, the code will still build fine on OS X, but will fail to link on
Linux.

We want to allow this code to be compiled on x86, albeit not run, to allow for
sharing of types between WASM and x86 programs in crates which depend on this.
*/
#[allow(clippy::too_many_arguments)]
#[allow(clippy::missing_safety_doc)]
#[cfg(not(target_arch = "wasm32"))]
mod ic0 {
    fn wrong_arch<A>(s: &str) -> A {
        panic!("{} should only be called inside canisters", s)
    }

    pub unsafe fn canister_self_copy(_dst: u32, _offset: u32, _size: u32) {
        wrong_arch("canister_self_copy")
    }
    pub unsafe fn canister_self_size() -> u32 {
        wrong_arch("canister_self_size")
    }
    pub unsafe fn debug_print(_offset: u32, _size: u32) {
        wrong_arch("debug_print")
    }
    pub unsafe fn msg_arg_data_copy(_dst: u32, _offset: u32, _size: u32) {
        wrong_arch("msg_arg_data_copy")
    }
    pub unsafe fn msg_arg_data_size() -> u32 {
        wrong_arch("canister_self_copy")
    }
    pub unsafe fn msg_caller_copy(_dst: u32, _offset: u32, _size: u32) {
        wrong_arch("msg_caller_copy")
    }
    pub unsafe fn msg_caller_size() -> u32 {
        wrong_arch("msg_caller_size")
    }
    pub unsafe fn msg_reject(_src: u32, _size: u32) {
        wrong_arch("msg_reject")
    }
    pub unsafe fn msg_reject_code() -> i32 {
        wrong_arch("msg_reject_code")
    }
    pub unsafe fn msg_reject_msg_copy(_dst: u32, _offset: u32, _size: u32) {
        wrong_arch("msg_reject_msg_copy")
    }
    pub unsafe fn msg_reject_msg_size() -> u32 {
        wrong_arch("msg_reject_msg_size")
    }
    pub unsafe fn msg_reply() {
        wrong_arch("msg_reply")
    }
    pub unsafe fn msg_reply_data_append(_offset: u32, _size: u32) {
        wrong_arch("msg_reply_data_append")
    }
    pub unsafe fn trap(_offset: u32, _size: u32) {
        wrong_arch("trap")
    }
    pub unsafe fn call_simple(
        _callee_src: u32,
        _callee_size: u32,
        _name_src: u32,
        _name_size: u32,
        _reply_fun: usize,
        _reply_env: u32,
        _reject_fun: usize,
        _reject_env: u32,
        _data_src: u32,
        _data_size: u32,
    ) -> i32 {
        wrong_arch("call_simple")
    }

    #[allow(dead_code)]
    pub unsafe fn stable_size() -> u32 {
        wrong_arch("stable_size")
    }

    #[allow(dead_code)]
    pub unsafe fn ic0_stable_grow(_additional_pages: u32) -> i32 {
        wrong_arch("stable_grow")
    }

    #[allow(dead_code)]
    pub fn ic0_stable_read(_dst: u32, _offset: u32, _size: u32) {
        wrong_arch("stable_read")
    }

    #[allow(dead_code)]
    pub fn ic0_stable_write(_offset: u32, _src: u32, _size: u32) {
        wrong_arch("stable_write")
    }
}

// Convenience wrappers around the DFINTY System API

/// Calls another canister and executes one of the callbacks.
pub fn call_with_callbacks(
    id: CanisterId,
    method: &str,
    data: &[u8],
    reply: impl FnOnce() + 'static,
    reject: impl FnOnce() + 'static,
) -> i32 {
    type Closures = (Box<dyn FnOnce() + 'static>, Box<dyn FnOnce() + 'static>);
    fn on_reply(env: u32) {
        let closure = unsafe { Box::from_raw(env as *mut Closures) }.0;
        closure();
    }
    fn on_reject(env: u32) {
        let closure = unsafe { Box::from_raw(env as *mut Closures) }.1;
        closure();
    }
    let callee = id.0;
    let boxed_closures: Box<Closures> = Box::new((Box::new(reply), Box::new(reject)));
    let env = Box::into_raw(boxed_closures);

    let err_code = unsafe {
        ic0::call_simple(
            callee.as_ptr() as u32,
            callee.len() as u32,
            method.as_ptr() as u32,
            method.len() as u32,
            on_reply as usize,
            env as u32,
            on_reject as usize,
            env as u32,
            data.as_ptr() as u32,
            data.len() as u32,
        )
    };

    if err_code != 0 {
        // deallocate the closures
        let _ = unsafe { Box::from_raw(env as *mut Closures) };
    }

    err_code
}

/// Calls another canister and returns a future.
pub fn call(id: CanisterId, method: &str, data: &[u8]) -> impl Future<Output = FutureResult> {
    // the callback from IC dereferences the future from a raw pointer, assigns the
    // result and calls the waker
    fn callback(future_ptr: u32) {
        let waker = {
            let ref_counted =
                unsafe { RefCounted::from_raw(future_ptr as *const RefCell<CallFuture>) };
            let mut future = ref_counted.borrow_mut();
            future.result = Some(match reject_code() {
                0 => Ok(arg_data()),
                n => Err((n, reject_message())),
            });
            future.waker.clone()
        };
        waker.expect("there is a waker").wake();
    };
    let callee = id.0;
    let future_for_closure = RefCounted::new(CallFuture::new());
    let future = future_for_closure.clone();
    let future_ptr = future_for_closure.into_raw();
    let err_code = unsafe {
        ic0::call_simple(
            callee.as_ptr() as u32,
            callee.len() as u32,
            method.as_ptr() as u32,
            method.len() as u32,
            callback as usize,
            future_ptr as u32,
            callback as usize,
            future_ptr as u32,
            data.as_ptr() as u32,
            data.len() as u32,
        )
    };
    // 0 is a special error code, meaning call_simple call succeeded
    if err_code != 0 {
        // Decrease the refcount as the closure will not be called.
        unsafe { RefCounted::from_raw(future_ptr) };
        future.borrow_mut().result = Some(Err((err_code, "Couldn't send message".to_string())));
    }
    future
}

/// Returns the argument extracted from the message payload.
pub fn arg_data() -> Vec<u8> {
    let len: u32 = unsafe { ic0::msg_arg_data_size() };
    let mut bytes = vec![0; len as usize];
    unsafe {
        ic0::msg_arg_data_copy(bytes.as_mut_ptr() as u32, 0, len);
    }
    bytes
}

/// Returns the caller of the current call.
pub fn caller() -> Vec<u8> {
    let len: u32 = unsafe { ic0::msg_caller_size() };
    let mut bytes = vec![0; len as usize];
    unsafe {
        ic0::msg_caller_copy(bytes.as_mut_ptr() as u32, 0, len);
    }
    bytes
}

/// Returns the canister id as a blob.
pub fn id() -> CanisterId {
    let len: u32 = unsafe { ic0::canister_self_size() };
    let mut bytes = vec![0; len as usize];
    unsafe {
        ic0::canister_self_copy(bytes.as_mut_ptr() as u32, 0, len);
    }
    CanisterId(bytes)
}

/// Returns the rejection message.
pub fn reject_message() -> String {
    let len: u32 = unsafe { ic0::msg_reject_msg_size() };
    let mut bytes = vec![0; len as usize];
    unsafe {
        ic0::msg_reject_msg_copy(bytes.as_mut_ptr() as u32, 0, len);
    }
    String::from_utf8_lossy(&bytes).to_string()
}

/// Replies with the given byte array.
/// Note, currently we do not support chunkwise assemlbing of the response.
pub fn reply(payload: &[u8]) {
    unsafe {
        ic0::msg_reply_data_append(payload.as_ptr() as u32, payload.len() as u32);
        ic0::msg_reply();
    }
}

/// Rejects the current call with the given message.
pub fn reject(err_message: &str) {
    let err_message = err_message.as_bytes();
    unsafe {
        ic0::msg_reject(err_message.as_ptr() as u32, err_message.len() as u32);
    }
}

/// Returns the rejection code.
pub fn reject_code() -> i32 {
    unsafe { ic0::msg_reject_code() }
}

/// Prints the given message.
pub fn print<S: std::convert::AsRef<str>>(s: S) {
    let s = s.as_ref();
    unsafe {
        ic0::debug_print(s.as_ptr() as u32, s.len() as u32);
    }
}

/// Traps with the given message.
pub fn trap_with(message: &str) {
    unsafe {
        ic0::trap(message.as_ptr() as u32, message.len() as u32);
    }
}

// This module contains all mechanisms required to enable asynchronous
// programming in Rust, based on native async Rust capabilities:
//
//  - the future returned by the asynchronous System API call, and
//  - the kickstarting/waker implementations to advance top level futures on
//    every inter-canister callback call.

use std::{
    cell::RefMut,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};

/// A reference counter wrapper we use with the CallFuture.
/// This is required, because the future we return from the `call` method can
/// either have two owners (the callback closure and the canister runtime) if
/// the underlying system call succeeded, or just one (the canister runtime) it
/// the system call failed.
pub struct RefCounted<T>(Rc<RefCell<T>>);

impl<T> RefCounted<T> {
    pub fn new(val: T) -> Self {
        RefCounted(Rc::new(RefCell::new(val)))
    }
    pub fn into_raw(self) -> *const RefCell<T> {
        Rc::into_raw(self.0)
    }
    pub unsafe fn from_raw(ptr: *const RefCell<T>) -> Self {
        Self(Rc::from_raw(ptr))
    }
    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        self.0.borrow_mut()
    }
}

impl<O, T: Future<Output = O>> Future for RefCounted<T> {
    type Output = O;
    fn poll(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe { Pin::new_unchecked(&mut *self.0.borrow_mut()) }.poll(ctx)
    }
}

impl<T> Clone for RefCounted<T> {
    fn clone(&self) -> Self {
        RefCounted(Rc::clone(&self.0))
    }
}

/// The result type of the CallFuture.
pub(super) type FutureResult = Result<Vec<u8>, (i32, String)>;

/// The Future trait implemenation, returned by the asynchronous inter-canister
/// call.
#[derive(Default)]
pub(super) struct CallFuture {
    /// result of the canister call
    pub result: Option<FutureResult>,
    /// waker (callback)
    pub waker: Option<Waker>,
}

impl CallFuture {
    pub fn new() -> Self {
        CallFuture::default()
    }
}

impl Future for CallFuture {
    type Output = FutureResult;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(result) = self.result.take() {
            return Poll::Ready(result);
        }
        self.waker = Some(cx.waker().clone());
        Poll::Pending
    }
}

/// Must be called on every top-level future corresponding to a method call of a
/// canister by the IC.
///
/// Saves the pointer to the future on the heap and kickstarts the future by
/// polling it once. During the polling we also need to provide the waker
/// callback which is triggered after the future made progress. The waker would
/// then. The waker would then poll the future one last time to advance it to
/// the final state. For that, we pass the future pointer to the waker, so that
/// it can be restored into a box from a raw pointer and then dropped if not
/// needed anymore.
///
/// Technically, we store 2 pointers on the heap: the pointer to the future
/// itself, and a pointer to that pointer. The reason for this is that the waker
/// API requires us to pass one thin pointer, while a a pointer to a `dyn Trait`
/// can only be fat. So we create one additional thin pointer, pointing to the
/// fat pointer and pass it instead.
pub fn kickstart<F: 'static + Future<Output = ()>>(future: F) {
    let future_ptr = Box::into_raw(Box::new(future));
    let future_ptr_ptr: *mut *mut dyn Future<Output = ()> = Box::into_raw(Box::new(future_ptr));
    let mut pinned_future = unsafe { Pin::new_unchecked(&mut *future_ptr) };
    if let Poll::Ready(_) = pinned_future
        .as_mut()
        .poll(&mut Context::from_waker(&waker::waker(
            future_ptr_ptr as *const (),
        )))
    {
        unsafe {
            let _ = Box::from_raw(future_ptr);
            let _ = Box::from_raw(future_ptr_ptr);
        }
    }
}

// This module conatins the implementation of a waker we're using for waking
// top-level futures (the ones returned by canister methods). The waker polls
// the future once and re-pins it on the heap, if it's pending. If the future is
// done, we do nothing. Hence, it will be deallocated once we exit the scope and
// we're not interested in the result, as it can only be a unit `()` if the
// waker was used as intended.
mod waker {
    use super::*;
    use std::task::{RawWaker, RawWakerVTable, Waker};
    type FuturePtr = *mut dyn Future<Output = ()>;

    static MY_VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

    fn raw_waker(ptr: *const ()) -> RawWaker {
        RawWaker::new(ptr, &MY_VTABLE)
    }

    fn clone(ptr: *const ()) -> RawWaker {
        raw_waker(ptr)
    }

    // Our waker will be called only if one of the response callbacks is triggered.
    // Then, the waker will restore the future from the pointer we passed into the
    // waker inside the `kickstart` method and poll the future again. If the future
    // is pending, we leave it on the heap. If it's ready, we deallocate the
    // pointer.
    unsafe fn wake(ptr: *const ()) {
        let boxed_future_ptr_ptr = Box::from_raw(ptr as *mut FuturePtr);
        let future_ptr: FuturePtr = *boxed_future_ptr_ptr;
        let boxed_future = Box::from_raw(future_ptr);
        let mut pinned_future = Pin::new_unchecked(&mut *future_ptr);
        if let Poll::Pending = pinned_future
            .as_mut()
            .poll(&mut Context::from_waker(&waker::waker(ptr)))
        {
            Box::into_raw(boxed_future_ptr_ptr);
            Box::into_raw(boxed_future);
        }
    }

    fn wake_by_ref(_: *const ()) {}

    fn drop(_: *const ()) {}

    pub fn waker(ptr: *const ()) -> Waker {
        unsafe { Waker::from_raw(raw_waker(ptr)) }
    }
}

/// Useful for writing endpoints that take a set of bytes and return a set of
/// bytes Check test/reverse_blob_bin.rs for a usage example
pub fn bytes<F: FnOnce(Vec<u8>) -> Vec<u8>>(f: F) {
    let bs = arg_data();
    let res = f(bs);
    reply(&res);
}
