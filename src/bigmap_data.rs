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
            "BigMap Data {}: get key {} ({} bytes) => value ({} bytes)",
            bm_data.canister_id(),
            String::from_utf8_lossy(&key),
            key.len(),
            value.len()
        ),
        None => println!(
            "BigMap Data {}: get key {} ({} bytes) => None",
            bm_data.canister_id(),
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
            "BigMap Data {}: get_as_update key {} ({} bytes) => value ({} bytes)",
            bm_data.canister_id(),
            String::from_utf8_lossy(&key),
            key.len(),
            value.len()
        ),
        None => println!(
            "BigMap Data {}: get_as_update key {} ({} bytes) => None",
            bm_data.canister_id(),
            String::from_utf8_lossy(&key),
            key.len()
        ),
    };
    res
}

#[update]
async fn put(key: Key, value: Val) -> bool {
    let bm_data = storage::get_mut::<DataBucket>();

    println!(
        "BigMap Data {}: put key {} ({} bytes) value ({} bytes)",
        bm_data.canister_id(),
        String::from_utf8_lossy(&key),
        key.len(),
        value.len()
    );
    bm_data.put(key, value);
    true
}

#[update]
async fn put_from_index(key_value: (Key, Val)) -> bool {
    // There is an ugly bug at the moment, where arguments in
    // a function call function(arg1, arg2) from
    // a Canister A to Canister B get converted into function((arg1, arg2))
    // in the target canister.
    // Therefore, we do the splitting of the arguments here.
    let (key, value) = key_value;
    put(key, value).await
}

#[update]
async fn reset() {
    let bm_data = storage::get::<DataBucket>();

    println!(
        "BigMap Data {}: FIXME: implement reset",
        bm_data.canister_id()
    );
}

#[query]
async fn holds_key(key: Key) -> Result<bool, String> {
    let bm_data = storage::get::<DataBucket>();

    Ok(bm_data.holds_key(&key))
}

#[query]
async fn used_bytes(_: ()) -> Result<usize, String> {
    let bm_data = storage::get::<DataBucket>();

    Ok(bm_data.used_bytes())
}

#[update]
async fn pop_entries_for_canister_id(can_id: CanisterId) -> Vec<(Key, Val)> {
    let bm_data = storage::get::<DataBucket>();

    let res: Vec<(Key, Val)> = Vec::new();

    println!(
        "BigMap Data {}: FIXME: implement pop_entries_for_canister_id {}",
        bm_data.canister_id(),
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
    //         "BigMap Data {}: key {} should be moved to canister_id={}",
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
    let bm_data = storage::get_mut::<DataBucket>();

    bm_data.set_index_canister(bigmap_idx_can);
    Ok(())
}

#[init]
fn initialize() {
    let bm_data = storage::get_mut::<DataBucket>();

    let can_id = ic_cdk::reflection::id().into();
    println!("BigMap Data {}: initialize", can_id);
    bm_data.set_canister_id(can_id);
}

fn main() {}
