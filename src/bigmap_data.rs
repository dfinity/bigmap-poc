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
            "BigMap Data: get key {} ({} bytes) => value ({} bytes)",
            String::from_utf8_lossy(&key),
            key.len(),
            value.len()
        ),
        None => println!(
            "BigMap Data: get key {} ({} bytes) => None",
            String::from_utf8_lossy(&key),
            key.len()
        ),
    };
    res
}

#[update]
fn put(key: Key, value: Val) -> u64 {
    let bm_data = storage::get_mut::<DataBucket>();

    let key_str = String::from_utf8_lossy(&key);
    println!(
        "BigMap Data: put key {} ({} bytes) value ({} bytes)",
        key_str,
        key.len(),
        value.len()
    );
    match bm_data.put(&key, &value, false) {
        Ok(value_len) => value_len,
        Err(err) => {
            println!("BigMap Data: put key {} error: {}", key_str, err);
            0
        }
    }
}

#[update]
// Returns the number of successful puts
fn batch_put(batch: Vec<(Key, Val)>) -> u64 {
    let bm_data = storage::get_mut::<DataBucket>();

    if batch.len() == 1 {
        let (key, value) = batch.get(0).unwrap();
        let key_str = String::from_utf8_lossy(&key);
        println!(
            "BigMap Data: put key {} ({} bytes) value ({} bytes)",
            key_str,
            key.len(),
            value.len()
        );

        match bm_data.put(&key, &value, false) {
            Ok(_) => 1,
            Err(err) => {
                println!("BigMap Data: put key {} error: {}", key_str, err);
                0
            }
        }
    } else {
        println!("BigMap Data: put batch of {} entries", batch.len());

        bm_data.batch_put(&batch)
    }
}

#[update]
fn append(key: Key, value: Val) -> u64 {
    let bm_data = storage::get_mut::<DataBucket>();

    let key_str = String::from_utf8_lossy(&key);
    let appended_value_len = value.len();
    match bm_data.put(&key, &value, true) {
        Ok(total_value_len) => {
            println!(
                "BigMap Data: put_append key {} ({} bytes) value ({} bytes appended, {} bytes total)",
                key_str,
                key.len(),
                appended_value_len,
                total_value_len
            );
            total_value_len
        }
        Err(err) => {
            println!("BigMap Data: put key {} error: {}", key_str, err);
            0
        }
    }
}

#[update]
fn put_from_index(key_value: (Key, Val)) -> u64 {
    // There is an ugly bug at the moment, where arguments in
    // a function call function(arg1, arg2) from
    // a Canister A to Canister B get converted into function((arg1, arg2))
    // in the target canister.
    // Therefore, we do the splitting of the arguments here.
    let (key, value) = key_value;
    put(key, value)
}

#[update]
fn append_from_index(key_value: (Key, Val)) -> u64 {
    let (key, value) = key_value;
    append(key, value)
}

#[update]
fn delete(key: Key) -> u64 {
    let bm_data = storage::get_mut::<DataBucket>();

    let key_str = String::from_utf8_lossy(&key);
    match bm_data.delete(key.clone()) {
        Ok(deleted_value_len) => {
            println!(
                "BigMap Data: delete key {} ({} bytes)",
                key_str, deleted_value_len
            );
            deleted_value_len
        }
        Err(err) => {
            println!("BigMap Data: delete key {} error: {}", key_str, err);
            0
        }
    }
}

#[query]
fn list(key_prefix: Key) -> Vec<Key> {
    let bm_data = storage::get::<DataBucket>();

    bm_data.list(&key_prefix)
}

#[query]
fn holds_key(key: Key) -> bool {
    let bm_data = storage::get::<DataBucket>();

    bm_data.holds_key(&key)
}

#[query]
fn used_bytes(_: ()) -> u64 {
    let bm_data = storage::get::<DataBucket>();

    bm_data.used_bytes() as u64
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
fn put_relocation_batch(batch: Vec<(Sha2Vec, Key, Val)>) -> u64 {
    let bm_data = storage::get_mut::<DataBucket>();

    bm_data.put_relocation_batch(&batch)
}

#[update]
fn delete_entries(keys_sha2: Vec<Vec<u8>>) {
    let bm_data = storage::get_mut::<DataBucket>();

    bm_data.delete_entries(&keys_sha2)
}

#[query]
fn get_random_key() -> String {
    let bm_data = storage::get::<DataBucket>();

    bm_data.get_random_key(None)
}

#[update]
fn seed_random_data(num_entries: u32, entry_size_bytes: u32) -> Vec<String> {
    let bm_data = storage::get_mut::<DataBucket>();

    bm_data.seed_random_data(num_entries, entry_size_bytes)
}

#[init]
fn initialize() {
    let bm_data = storage::get_mut::<DataBucket>();

    println!("BigMap Data: initialize");
    let can_id = ic_cdk::reflection::id().into();
    bm_data.set_canister_id(can_id);
}

fn main() {}
