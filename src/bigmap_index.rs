use ::bigmap::{index::BigmapIdx, CanisterId, Key, Val};
#[cfg(target_arch = "wasm32")]
use ic_cdk::println;
use ic_cdk::storage;
use ic_cdk_macros::*;

#[query]
async fn get(key: Key) -> Option<Val> {
    let bigmap_idx = storage::get::<BigmapIdx>();

    match bigmap_idx.lookup_get(&key).await {
        Some(can_id) => {
            let can_id: ic_cdk::CanisterId = can_id.0.into();
            println!(
                "BigMap Index: get key {} @CanisterId {}",
                String::from_utf8_lossy(&key),
                can_id
            );
            ic_cdk::call(can_id, "get", Some(key)).await.unwrap()
        }
        None => {
            println!(
                "BigMap Index: no data canister holds the key {}",
                String::from_utf8_lossy(&key)
            );
            None
        }
    }
}

#[update]
async fn put(key: Key, value: Val) -> u64 {
    let bigmap_idx = storage::get::<BigmapIdx>();

    match bigmap_idx.lookup_put(&key) {
        Some(can_id) => {
            let can_id: ic_cdk::CanisterId = can_id.0.into();
            println!(
                "BigMap Index: put key {} @CanisterId {}",
                String::from_utf8_lossy(&key),
                can_id
            );
            ic_cdk::call(can_id.clone(), "put_from_index", Some((key, value)))
                .await
                .expect(&format!(
                    "BigMap index: put call to CanisterId {} failed",
                    can_id
                ))
        }
        None => {
            println!(
                "BigMap Index: no data canister suitable for key {}",
                String::from_utf8_lossy(&key)
            );
            0
        }
    }
}

#[update]
async fn append(key: Key, value: Val) -> usize {
    let bigmap_idx = storage::get::<BigmapIdx>();

    match bigmap_idx.lookup_put(&key) {
        Some(can_id) => {
            let can_id: ic_cdk::CanisterId = can_id.0.into();
            println!(
                "BigMap Index: append key {} @CanisterId {}",
                String::from_utf8_lossy(&key),
                can_id
            );
            ic_cdk::call(can_id.clone(), "append_from_index", Some((key, value)))
                .await
                .expect(&format!(
                    "BigMap index: append call to CanisterId {} failed",
                    can_id
                ))
        }
        None => {
            println!(
                "BigMap Index: no data canister suitable for key {}",
                String::from_utf8_lossy(&key)
            );
            0
        }
    }
}

#[query]
fn needs_data_buckets() -> u32 {
    let bigmap_idx = storage::get::<BigmapIdx>();

    bigmap_idx.canisters_needed()
}

#[update]
async fn add_data_buckets(can_vec: Vec<String>) {
    let bigmap_idx = storage::get_mut::<BigmapIdx>();

    let mut cans: Vec<CanisterId> = Vec::new();
    for can_text in can_vec {
        let can_id = ic_cdk::CanisterId::from_str(&can_text).unwrap();
        cans.push(can_id.into());
    }
    bigmap_idx.add_canisters(cans).await;
}

#[query]
async fn lookup_data_bucket_for_put(key: Key) -> Option<String> {
    let bigmap_idx = storage::get::<BigmapIdx>();

    match bigmap_idx.lookup_put(&key) {
        Some(can_id) => Some(format!("{}", can_id)),
        None => None,
    }
}

#[query]
async fn lookup_data_bucket_for_get(key: Key) -> Option<String> {
    let bigmap_idx = storage::get_mut::<BigmapIdx>();

    match bigmap_idx.lookup_get(&key).await {
        Some(can_id) => {
            let can_id = format!("{}", can_id);
            println!(
                "BigMap Index: lookup_data_bucket_for_get key {} => {}",
                String::from_utf8_lossy(&key),
                can_id
            );
            Some(can_id)
        }
        None => None,
    }
}

#[query]
async fn get_random_key() -> String {
    let bigmap_idx = storage::get::<BigmapIdx>();

    bigmap_idx.get_random_key().await
}

#[update]
fn set_used_bytes_threshold(threshold: u32) {
    let bigmap_idx = storage::get_mut::<BigmapIdx>();

    bigmap_idx.set_used_bytes_threshold(threshold);
}

#[update]
async fn maintenance() -> String {
    let bigmap_idx = storage::get_mut::<BigmapIdx>();

    bigmap_idx.maintenance().await
}

#[init]
fn initialize() {
    let bigmap_idx = storage::get_mut::<BigmapIdx>();

    let can_id = ic_cdk::reflection::id().into();
    println!("BigMap Index: initialize");
    bigmap_idx.reset();
    bigmap_idx.set_canister_id(can_id);
    ic_cdk::setup();
}

#[query]
async fn total_used_bytes() -> usize {
    0
}

#[update]
fn set_data_bucket_canister_wasm_binary(wasm_binary: Vec<u8>) {
    let bigmap_idx = storage::get_mut::<BigmapIdx>();
    println!(
        "BigMap Index: set_data_bucket_canister_wasm_binary ({} bytes)",
        wasm_binary.len()
    );

    bigmap_idx.set_data_bucket_canister_wasm_binary(wasm_binary)
}

fn main() {}
