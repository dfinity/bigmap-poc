use ::bigmap::{index::BigmapIdx, println, CanisterId, Key};
use futures::executor::block_on;
use ic_cdk_macros::*;
use lazy_static::lazy_static;
use std::sync::Mutex;

#[query]
#[allow(dead_code)]
async fn needs_data_buckets() -> Result<u32, String> {
    let bigmap_idx = match (&*BM_IDX).lock() {
        Ok(v) => v,
        Err(err) => return Err(format!("Failed to lock the Bigmap Idx, err: {}", err)),
    };

    Ok(bigmap_idx.canisters_needed())
}

#[update]
#[allow(dead_code)]
async fn add_data_buckets(can_vec: Vec<CanisterId>) -> Result<(), String> {
    let mut bigmap_idx = match (&*BM_IDX).lock() {
        Ok(v) => v,
        Err(err) => return Err(format!("Failed to lock the Bigmap Idx, err: {}", err)),
    };
    println!("BigMap Index: add data canister_id vec={:?}", can_vec);

    bigmap_idx.add_canisters(can_vec)?;
    Ok(())
}

#[query]
#[allow(dead_code)]
async fn key_to_canister_id(key: Key) -> Result<Vec<CanisterId>, String> {
    let bigmap_idx = match (&*BM_IDX).lock() {
        Ok(v) => v,
        Err(err) => return Err(format!("Failed to lock the Bigmap Idx, err: {}", err)),
    };

    bigmap_idx.lookup(&key)
}

#[update]
#[allow(dead_code)]
async fn rebalance(_: ()) -> Result<u8, String> {
    let mut bigmap_idx = match (&*BM_IDX).lock() {
        Ok(v) => v,
        Err(err) => return Err(format!("Failed to lock the Bigmap Idx, err: {}", err)),
    };

    bigmap_idx.rebalance()
}

#[init]
fn initialize() {
    let mut bigmap_idx = match (&*BM_IDX).lock() {
        Ok(v) => v,
        Err(err) => {
            println!("BigMap Index: failed to lock the BM_IDX");
            return;
        }
    };

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
