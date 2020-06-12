//! This module contains all mechanisms required to enable asynchronous
//! programming in Rust, based on native async Rust capabilities:
//!
//!  - the future returned by the asynchronous System API call, and
//!  - the kickstarting/waker implementations to advance top level futures on
//!    every inter-canister callback call.

use std::{
    cell::{RefCell, RefMut},
    future::Future,
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
