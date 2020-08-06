use ::bigmap::call_candid;
use ::bigmap::{index::BigmapIdx, CanisterId, Key, Val};
// use futures::executor::block_on;
#[cfg(target_arch = "wasm32")]
use ic_cdk::println;
use ic_cdk::storage;
use ic_cdk_macros::*;
// use lazy_static::lazy_static;
// use std::sync::Mutex;

#[update]
#[allow(dead_code)]
async fn get(key: Key) -> Option<Val> {
    let bigmap_idx = storage::get::<BigmapIdx>();

    match bigmap_idx.lookup_get(&key) {
        Some(can_id) => {
            let can_id: ic_cdk::CanisterId = can_id.0.into();
            println!(
                "BigMap Index {}: get key {} @CanisterId {}",
                bigmap_idx.canister_id(),
                String::from_utf8_lossy(&key),
                can_id
            );
            // ic_cdk::call(can_id, "get_as_update", Some(key))
            //     .await
            //     .unwrap()
            call_candid(can_id.0, "get_as_update", key).await.unwrap()
        }
        None => {
            println!(
                "BigMap Index {}: no data canister suitable for key {}",
                bigmap_idx.canister_id(),
                String::from_utf8_lossy(&key)
            );
            None
        }
    }
}

#[update]
#[allow(dead_code)]
async fn put(key: Key, value: Val) -> bool {
    let bigmap_idx = storage::get::<BigmapIdx>();

    match bigmap_idx.lookup_put(&key) {
        Some(can_id) => {
            let can_id: ic_cdk::CanisterId = can_id.0.into();
            println!(
                "BigMap Index {}: put key {} @CanisterId {}",
                bigmap_idx.canister_id(),
                String::from_utf8_lossy(&key),
                can_id
            );
            ic_cdk::call(can_id.clone(), "put", Some((key, value)))
                .await
                .expect(&format!(
                    "BigMap index: put call to CanisterId {} failed",
                    can_id
                ))
        }
        None => {
            println!(
                "BigMap Index {}: no data canister suitable for key {}",
                bigmap_idx.canister_id(),
                String::from_utf8_lossy(&key)
            );
            false
        }
    }
}

#[query]
#[allow(dead_code)]
fn needs_data_buckets() -> u32 {
    let bigmap_idx = storage::get::<BigmapIdx>();

    bigmap_idx.canisters_needed()
}

#[update]
#[allow(dead_code)]
fn add_data_buckets(can_vec: Vec<String>) {
    let bigmap_idx = storage::get_mut::<BigmapIdx>();

    let mut cans: Vec<CanisterId> = Vec::new();
    for can_text in can_vec {
        let can_id = ic_cdk::CanisterId::from_str(&can_text).unwrap();
        cans.push(can_id.into());
    }
    bigmap_idx.add_canisters(cans);
}

#[query]
#[allow(dead_code)]
async fn lookup_data_bucket_for_put(key: Key) -> Option<String> {
    let bigmap_idx = storage::get::<BigmapIdx>();

    match bigmap_idx.lookup_put(&key) {
        Some(can_id) => Some(format!("{}", can_id)),
        None => None,
    }
}

#[query]
#[allow(dead_code)]
async fn lookup_data_bucket_for_get(key: Key) -> Option<String> {
    let bigmap_idx = storage::get_mut::<BigmapIdx>();

    match bigmap_idx.lookup_get(&key) {
        Some(can_id) => {
            let can_id = format!("{}", can_id);
            println!(
                "BigMap Index {}: lookup_data_bucket_for_get key {} => {}",
                bigmap_idx.canister_id(),
                String::from_utf8_lossy(&key),
                can_id
            );
            Some(can_id)
        }
        None => None,
    }
}

#[init]
fn initialize() {
    let bigmap_idx = storage::get_mut::<BigmapIdx>();

    let can_id = ic_cdk::reflection::id().into();
    println!("BigMap Index {}: initialize", can_id);
    bigmap_idx.set_canister_id(can_id);
    bigmap_idx.set_fn_xcq_used_bytes(Box::new(xcq_used_bytes_fn));
    bigmap_idx.set_fn_xcq_holds_key(Box::new(xcq_holds_key_fn));
}

#[query]
async fn total_used_bytes() -> usize {
    0
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

// lazy_static! {
//     static ref BM_IDX: Mutex<BigmapIdx> = Mutex::new(BigmapIdx::new());
// }

fn main() {}
