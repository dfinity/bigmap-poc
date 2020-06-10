use ::bigmap::println;
use ic_cdk_macros::*;

static mut COUNTER: Option<candid::Nat> = None;

#[init]
fn init() {
    unsafe {
        COUNTER = Some(candid::Nat::from(0));
    }
    let my_id = ic_cdk::reflection::id();
    println!("Big Map Data Canister {} initialized", my_id);
}

#[update]
fn inc() -> () {
    unsafe {
        COUNTER.as_mut().unwrap().0 += 1u64;
    }
}

#[query]
fn read() -> candid::Nat {
    unsafe { COUNTER.as_mut().unwrap().clone() }
}

#[update]
fn write(input: candid::Nat) -> () {
    unsafe {
        COUNTER.as_mut().unwrap().0 = input.0;
    }
}

fn main() {}
