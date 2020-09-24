use ::bigmap::{index::BigmapIdx, CanisterId, Key, Val};
#[cfg(target_arch = "wasm32")]
use ic_cdk::println;
use ic_cdk::storage;
use ic_cdk_macros::*;

#[query]
async fn get(key: Key) -> Option<Val> {
    let bigmap_idx = storage::get::<BigmapIdx>();

    bigmap_idx.get(&key).await
}

#[update]
async fn put(key: Key, value: Val) -> u64 {
    let bigmap_idx = storage::get_mut::<BigmapIdx>();

    println!("BigMap Index: put key {}", String::from_utf8_lossy(&key));

    bigmap_idx.put(&key, &value).await
}

#[update]
// Returns the number of successful puts
async fn batch_put(batch: Vec<(Key, Val)>) -> u64 {
    let bigmap_idx = storage::get_mut::<BigmapIdx>();
    if batch.len() == 1 {
        let (key, value) = batch.get(0).unwrap();
        println!("BigMap Index: put key {}", String::from_utf8_lossy(key));

        bigmap_idx.put(key, value).await;
        1
    } else {
        println!("BigMap Data: put batch of {} entries", batch.len());

        bigmap_idx.batch_put(&batch).await
    }
}

#[update]
async fn append(key: Key, value: Val) -> u64 {
    let bigmap_idx = storage::get_mut::<BigmapIdx>();

    bigmap_idx.append(&key, &value).await
}

#[update]
async fn delete(key: Key) -> u64 {
    let bigmap_idx = storage::get::<BigmapIdx>();

    match bigmap_idx.lookup_put(&key) {
        Some(can_id) => {
            let can_id: ic_cdk::CanisterId = can_id.0.into();
            println!(
                "BigMap Index: delete key {} @CanisterId {}",
                String::from_utf8_lossy(&key),
                can_id
            );
            ic_cdk::call(can_id.clone(), "delete", Some(key))
                .await
                .expect(&format!(
                    "BigMap index: delete call to CanisterId {} failed",
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
async fn list(key_prefix: Key) -> Vec<Key> {
    let bigmap_idx = storage::get::<BigmapIdx>();

    bigmap_idx.list(&key_prefix).await
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

#[query]
async fn status() -> String {
    let bigmap_idx = storage::get::<BigmapIdx>();

    bigmap_idx.status().await
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

#[update]
async fn set_data_bucket_canister_wasm_binary(wasm_binary: Vec<u8>) {
    let bigmap_idx = storage::get_mut::<BigmapIdx>();
    println!(
        "BigMap Index: set_data_bucket_canister_wasm_binary ({} bytes)",
        wasm_binary.len()
    );

    bigmap_idx
        .set_data_bucket_canister_wasm_binary(wasm_binary)
        .await
}

#[update]
async fn set_search_canister_wasm_binary(wasm_binary: Vec<u8>) {
    let bigmap_idx = storage::get_mut::<BigmapIdx>();
    println!(
        "BigMap Index: set_search_canister_wasm_binary ({} bytes)",
        wasm_binary.len()
    );

    bigmap_idx
        .set_search_canister_wasm_binary(wasm_binary)
        .await
}

#[update]
async fn put_and_fts_index(key: Key, document: String) -> u64 {
    let bigmap_idx = storage::get_mut::<BigmapIdx>();

    if document.len() > 100 {
        println!(
            "BigMap Search Index: add key {} => document[0..100] {}",
            String::from_utf8_lossy(&key),
            &document[0..100]
        );
    } else {
        println!(
            "BigMap Search Index: add key {} => document {}",
            String::from_utf8_lossy(&key),
            document
        );
    }

    bigmap_idx.put_and_fts_index(&key, &document).await
}

#[update]
async fn batch_put_and_fts_index(doc_vec: Vec<(Key, String)>) -> u64 {
    let bigmap_idx = storage::get_mut::<BigmapIdx>();

    bigmap_idx.batch_put_and_fts_index(&doc_vec).await
}

#[update]
async fn remove_from_fts_index(key: Key) {
    let bigmap_idx = storage::get_mut::<BigmapIdx>();

    println!(
        "BigMap Search Index: remove key {}",
        String::from_utf8_lossy(&key)
    );

    bigmap_idx.remove_from_fts_index(&key).await
}

#[query]
async fn search(query: String) -> (u64, Vec<(Key, Val)>) {
    let bigmap_idx = storage::get_mut::<BigmapIdx>();

    bigmap_idx.search(&query).await
}

fn main() {}
