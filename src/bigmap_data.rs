use ::bigmap::data::DataBucket;
use ::bigmap::{CanisterId, Key, Val};
#[cfg(target_arch = "wasm32")]
use ic_cdk::println;
use ic_cdk::storage;
use ic_cdk_macros::*;
// use lazy_static::lazy_static;
// use std::sync::Mutex;

#[query]
async fn get(key: Key) -> Option<Val> {
    let bm_data = storage::get::<DataBucket>();

    let res = bm_data.get(key.clone()).ok().cloned();
    match &res {
        Some(value) => println!(
            "BigMap data: get key {} ({} bytes) => value ({} bytes)",
            String::from_utf8_lossy(&key),
            key.len(),
            value.len()
        ),
        None => println!(
            "BigMap data: get key {} ({} bytes) => None",
            String::from_utf8_lossy(&key),
            key.len()
        ),
    };
    res
}

#[update]
async fn get_as_update(key: Key) -> Option<Val> {
    let bm_data = storage::get::<DataBucket>();

    let res = bm_data.get(key.clone()).ok().cloned();
    match &res {
        Some(value) => println!(
            "BigMap data: get_as_update key {} ({} bytes) => value ({} bytes)",
            String::from_utf8_lossy(&key),
            key.len(),
            value.len()
        ),
        None => println!(
            "BigMap data: get_as_update key {} ({} bytes) => None",
            String::from_utf8_lossy(&key),
            key.len()
        ),
    };
    res
}

#[update]
async fn put(key_value: (Key, Val)) -> bool {
    let bm_data = storage::get::<DataBucket>();

    let (key, value) = key_value;
    println!(
        "BigMap data: put key {} ({} bytes) value ({} bytes)",
        String::from_utf8_lossy(&key),
        key.len(),
        value.len()
    );
    bm_data.put(key, value);
    true
}

#[update]
async fn reset() {
    // let bm_data = storage::get::<DataBucket>();

    println!("BigMap data: FIXME: implement reset");
}

#[query]
#[allow(dead_code)]
async fn holds_key(key: Key) -> Result<bool, String> {
    let bm_data = storage::get::<DataBucket>();

    Ok(bm_data.holds_key(&key))
}

#[query]
#[allow(dead_code)]
async fn used_bytes(_: ()) -> Result<usize, String> {
    let bm_data = storage::get::<DataBucket>();

    Ok(bm_data.used_bytes())
}

#[update]
#[allow(dead_code)]
async fn pop_entries_for_canister_id(can_id: CanisterId) -> Vec<(Key, Val)> {
    // let bm_data = match (&*BM_DATA).lock() {
    //     Ok(v) => v,
    //     Err(_) => return Vec::new(),
    // };

    let res: Vec<(Key, Val)> = Vec::new();

    println!(
        "BigMap data: FIXME: implement pop_entries_for_canister_id {}",
        can_id
    );

    // let _keys: Vec<Key> = bm_data
    //     .entries
    //     .keys()
    //     .map(|v| v.clone())
    //     .collect::<Vec<Key>>();

    // let filt_keys: Vec<Vec<u8>> = call_json(
    //     bm_data.index_canister.clone(),
    //     "filter_keys_mapping_to_canister_id",
    //     (keys, can_id.clone()),
    // )
    // .await
    // .unwrap();

    // for k in filt_keys {
    //     println!(
    //         "BigMap data: key {} should be moved to canister_id={}",
    //         String::from_utf8_lossy(&k),
    //         can_id
    //     );
    //     res.push((k.clone(), bm_data.entries.remove(&k).unwrap()))
    // }

    res
}

#[update]
#[allow(dead_code)]
async fn set_bigmap_idx_can(bigmap_idx_can: CanisterId) -> Result<(), String> {
    let bm_data = storage::get::<DataBucket>();

    bm_data.set_index_canister(bigmap_idx_can);
    Ok(())
}

fn main() {}
