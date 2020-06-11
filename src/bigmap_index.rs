use ::bigmap::{index::BigmapIdx, println, CanisterId, Key};
use futures::executor::block_on;
use ic_cdk_macros::*;
use lazy_static::lazy_static;
use std::sync::Mutex;

#[query]
#[allow(dead_code)]
fn needs_data_buckets() -> u32 {
    let bigmap_idx = match (&*BM_IDX).lock() {
        Ok(v) => v,
        Err(err) => {
            println!("BigMap Index: failed to lock the Bigmap Idx, err: {}", err);
            return 0;
        }
    };

    bigmap_idx.canisters_needed()
}

#[update]
#[allow(dead_code)]
fn add_data_buckets(can_vec: Vec<CanisterId>) -> () {
    let mut bigmap_idx = match (&*BM_IDX).lock() {
        Ok(v) => v,
        Err(err) => {
            println!("BigMap Index: failed to lock the Bigmap Idx, err: {}", err);
            return;
        }
    };

    bigmap_idx.add_canisters(can_vec);
}

#[query]
#[allow(dead_code)]
async fn key_to_data_bucket(key: Key) -> CanisterId {
    let bigmap_idx = match (&*BM_IDX).lock() {
        Ok(v) => v,
        Err(err) => {
            println!("BigMap Index: failed to lock the Bigmap Idx, err: {}", err);
            return Default::default();
        }
    };

    bigmap_idx.lookup(&key)
}

#[init]
fn initialize() {
    let mut bigmap_idx = match (&*BM_IDX).lock() {
        Ok(v) => v,
        Err(err) => {
            println!("BigMap Index: failed to lock the Bigmap Idx, err: {}", err);
            return;
        }
    };

    println!("BigMap Index: initialize");
    bigmap_idx.set_canister_id(ic_cdk::reflection::id().into());
    bigmap_idx.set_fn_xcq_used_bytes(Box::new(xcq_used_bytes_fn));
    bigmap_idx.set_fn_xcq_holds_key(Box::new(xcq_holds_key_fn));
}

// Temporary function, until we're able to call other canisters from native code
// in Rust SDK
fn xcq_used_bytes_fn(can_id: CanisterId) -> usize {
    let res: Result<usize, String> =
        block_on(ic_cdk::call(can_id.0.into(), "used_bytes", None::<()>))
            .expect("async call failed");
    res.expect("used_bytes call failed")
}

// Temporary function, until we're able to call other canisters from native code
// in Rust SDK
fn xcq_holds_key_fn(can_id: CanisterId, key: &Key) -> bool {
    let res: Result<bool, String> =
        block_on(ic_cdk::call(can_id.0.into(), "holds_key", Some(key))).expect("async call failed");
    res.expect("holds_key call failed")
}

lazy_static! {
    static ref BM_IDX: Mutex<BigmapIdx> = Mutex::new(BigmapIdx::new());
}

fn main() {}
