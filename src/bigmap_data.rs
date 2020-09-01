use ::bigmap::data::DataBucket;
use ::bigmap::{Key, Sha2Vec, Val};
#[cfg(target_arch = "wasm32")]
use ic_cdk::println;
use ic_cdk::storage;
use ic_cdk_macros::*;
// use std::sync::Mutex;

#[query]
fn get(key: Key) -> Option<Val> {
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
fn get_as_update(key: Key) -> Option<Val> {
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
fn put(key: Key, value: Val) -> bool {
    let bm_data = storage::get_mut::<DataBucket>();

    let key_str = String::from_utf8_lossy(&key);
    println!(
        "BigMap Data {}: put key {} ({} bytes) value ({} bytes)",
        bm_data.canister_id(),
        key_str,
        key.len(),
        value.len()
    );
    match bm_data.put(key.clone(), value) {
        Ok(_) => true,
        Err(err) => {
            println!(
                "BigMap Data {}: put key {} error: {}",
                bm_data.canister_id(),
                key_str,
                err
            );
            false
        }
    }
}

#[update]
async fn put_from_index(key_value: (Key, Val)) -> bool {
    // There is an ugly bug at the moment, where arguments in
    // a function call function(arg1, arg2) from
    // a Canister A to Canister B get converted into function((arg1, arg2))
    // in the target canister.
    // Therefore, we do the splitting of the arguments here.
    let (key, value) = key_value;
    put(key, value)
}

#[update]
fn reset() {
    let bm_data = storage::get::<DataBucket>();

    println!(
        "BigMap Data {}: FIXME: implement reset",
        bm_data.canister_id()
    );
}

#[query]
fn holds_key(key: Key) -> bool {
    let bm_data = storage::get::<DataBucket>();

    bm_data.holds_key(&key)
}

#[query]
fn used_bytes(_: ()) -> usize {
    let bm_data = storage::get::<DataBucket>();

    bm_data.used_bytes()
}

#[update]
fn set_range(range: (Vec<u8>, Vec<u8>)) {
    let bm_data = storage::get_mut::<DataBucket>();

    let (mut range_start, mut range_end) = range;
    range_start.resize_with(32, Default::default);
    range_end.resize_with(32, Default::default);
    let range_start = generic_array::GenericArray::from_slice(&range_start);
    let range_end = generic_array::GenericArray::from_slice(&range_end);
    bm_data.set_range(range_start, range_end);
}

#[query]
fn get_relocation_batch(batch_limit_bytes: u64) -> Vec<(Sha2Vec, Key, Val)> {
    let bm_data = storage::get::<DataBucket>();

    bm_data.get_relocation_batch(batch_limit_bytes)
}

#[update]
fn put_batch(batch: Vec<(Sha2Vec, Key, Val)>) -> u64 {
    let bm_data = storage::get_mut::<DataBucket>();

    bm_data.put_batch(&batch)
}

#[update]
fn delete_entries(keys_sha2: Vec<Vec<u8>>) {
    let bm_data = storage::get_mut::<DataBucket>();

    bm_data.delete_entries(&keys_sha2)
}

#[init]
fn initialize() {
    let bm_data = storage::get_mut::<DataBucket>();

    let can_id = ic_cdk::reflection::id().into();
    println!("BigMap Data {}: initialize", can_id);
    bm_data.set_canister_id(can_id);
}

fn main() {}
