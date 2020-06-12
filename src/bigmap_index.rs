// use ::bigmap::call_candid;
use ::bigmap::{index::BigmapIdx, CanisterId, Key, Val};
// use futures::executor::block_on;
#[cfg(target_arch = "wasm32")]
use ic_cdk::println;
use ic_cdk_macros::*;
use lazy_static::lazy_static;
use std::sync::Mutex;

#[update]
#[allow(dead_code)]
async fn get(key: Key) -> Option<Val> {
    let bigmap_idx = match (&*BM_IDX).lock() {
        Ok(v) => v,
        Err(err) => {
            println!("BigMap Index: failed to lock the Bigmap Idx, err: {}", err);
            return None;
        }
    };

    println!("BigMap index: get key {}", String::from_utf8_lossy(&key));
    match bigmap_idx.lookup_get(&key) {
        Some(can_id) => {
            let can_id: ic_cdk::CanisterId = can_id.0.into();
            println!(
                "BigMap index: get key {} @CanisterId {}",
                String::from_utf8_lossy(&key),
                can_id
            );
            // FIXME: ic_ckd::call doesn't work
            ic_cdk::call(can_id, "get_as_update", Some(key))
                .await
                .unwrap()
            // call_candid(can_id.0, "get_as_update", key).await.unwrap()
        }
        None => {
            println!(
                "BigMap index: no data canister suitable for key {}",
                String::from_utf8_lossy(&key)
            );
            None
        }
    }
}

#[update]
#[allow(dead_code)]
async fn put(key: Key, value: Val) {
    let bigmap_idx = match (&*BM_IDX).lock() {
        Ok(v) => v,
        Err(err) => {
            println!("BigMap Index: failed to lock the Bigmap Idx, err: {}", err);
            return;
        }
    };

    println!(
        "BigMap index: put key {} val {}",
        String::from_utf8_lossy(&key),
        String::from_utf8_lossy(&value)
    );
    match bigmap_idx.lookup_put(&key) {
        Some(can_id) => {
            let can_id: ic_cdk::CanisterId = can_id.0.into();
            println!(
                "BigMap index: put key {} @CanisterId {}",
                String::from_utf8_lossy(&key),
                can_id
            );
            ic_cdk::call_no_return(can_id.clone(), "put", Some((key, value)))
                .await
                .expect(&format!(
                    "BigMap index: put call to CanisterId {} failed",
                    can_id
                ));
            // let x = bigmap::dfn_candid::from_output((key.clone(), value.clone()));
            // let y: (Key, Val) = bigmap::dfn_candid::to_input(x.clone());
            // println!("{:?}", x);
            // println!("{:?}, {:?}", y.0, y.1);
            // call_candid(can_id.0, "put", (key, value)).await.unwrap()
        }
        None => {
            println!(
                "BigMap index: no data canister suitable for key {}",
                String::from_utf8_lossy(&key)
            );
        }
    }
}

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
fn add_data_buckets(can_vec: Vec<String>) {
    let mut bigmap_idx = match (&*BM_IDX).lock() {
        Ok(v) => v,
        Err(err) => {
            println!("BigMap Index: failed to lock the Bigmap Idx, err: {}", err);
            return;
        }
    };

    let mut cans: Vec<CanisterId> = Vec::new();
    for can_text in can_vec {
        let can_id = ic_cdk::CanisterId::from_str_unchecked(&can_text).unwrap();
        cans.push(can_id.into());
    }
    bigmap_idx.add_canisters(cans);
}

#[query]
#[allow(dead_code)]
async fn lookup_data_bucket_for_put(key: Key) -> Option<String> {
    let bigmap_idx = match (&*BM_IDX).lock() {
        Ok(v) => v,
        Err(err) => {
            println!("BigMap Index: failed to lock the Bigmap Idx, err: {}", err);
            return Default::default();
        }
    };

    match bigmap_idx.lookup_put(&key) {
        Some(can_id) => Some(format!("{}", can_id)),
        None => None,
    }
}

#[query]
#[allow(dead_code)]
async fn lookup_data_bucket_for_get(key: Key) -> Option<String> {
    let bigmap_idx = match (&*BM_IDX).lock() {
        Ok(v) => v,
        Err(err) => {
            println!("BigMap Index: failed to lock the Bigmap Idx, err: {}", err);
            return Default::default();
        }
    };

    match bigmap_idx.lookup_get(&key) {
        Some(can_id) => Some(format!("{}", can_id)),
        None => None,
    }
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

// async fn xcq_used_bytes_async(can_id: CanisterId) -> usize {
//     ic_cdk::call(can_id.0.into(), "used_bytes", None::<()>)
//         .await
//         .expect("async call failed")
// }

// Temporary function, until we're able to call other canisters from native code
// in Rust SDK
fn xcq_used_bytes_fn(_can_id: CanisterId) -> usize {
    // ic_cdk::block_on(ic_cdk::call(can_id.0.into(), "used_bytes", None::<()>));
    // let res: Result<usize, String> = arg_data_1<usize>();
    // let res: usize = xcq_used_bytes_async(can_id.0.into());
    // res.expect("used_bytes call failed")
    // FIXME: do inter-canister calls
    0
}

// Temporary function, until we're able to call other canisters from native code
// in Rust SDK
fn xcq_holds_key_fn(_can_id: CanisterId, _key: &Key) -> bool {
    // let res = async_runtime::rt::spawn(ic_cdk::call(can_id.0.into(), "holds_key", Some(key)))
    //     .expect("async call failed");
    // res.expect("holds_key call failed")
    // FIXME: do inter-canister calls
    true
}

lazy_static! {
    static ref BM_IDX: Mutex<BigmapIdx> = Mutex::new(BigmapIdx::new());
}

fn main() {}
