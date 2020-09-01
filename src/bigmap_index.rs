use ::bigmap::{index::BigmapIdx, CanisterId, Key, Val};
#[cfg(target_arch = "wasm32")]
use ic_cdk::println;
use ic_cdk::storage;
use ic_cdk_macros::*;

#[update]
async fn get(key: Key) -> Option<Val> {
    let bigmap_idx = storage::get::<BigmapIdx>();

    match bigmap_idx.lookup_get(&key).await {
        Some(can_id) => {
            let can_id: ic_cdk::CanisterId = can_id.0.into();
            println!(
                "BigMap Index {}: get key {} @CanisterId {}",
                bigmap_idx.canister_id(),
                String::from_utf8_lossy(&key),
                can_id
            );
            ic_cdk::call(can_id, "get", Some(key)).await.unwrap()
            // call_candid(can_id.0, "get_as_update", key).await.unwrap()
        }
        None => {
            println!(
                "BigMap Index {}: no data canister holds the key {}",
                bigmap_idx.canister_id(),
                String::from_utf8_lossy(&key)
            );
            None
        }
    }
}

#[update]
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
            ic_cdk::call(can_id.clone(), "put_from_index", Some((key, value)))
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

#[update]
async fn maintenance() -> Result<u8, String> {
    let bigmap_idx = storage::get_mut::<BigmapIdx>();

    bigmap_idx.maintenance().await
}

#[init]
fn initialize() {
    let bigmap_idx = storage::get_mut::<BigmapIdx>();

    let can_id = ic_cdk::reflection::id().into();
    println!("BigMap Index {}: initialize", can_id);
    bigmap_idx.set_canister_id(can_id);
}

#[query]
async fn total_used_bytes() -> usize {
    0
}

fn main() {}
