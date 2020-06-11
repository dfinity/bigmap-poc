use ::bigmap::data::DataBucket;
use ::bigmap::{println, CanisterId, Key, Val};
use ic_cdk_macros::*;
use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref BM_DATA: Mutex<DataBucket> = Mutex::new(DataBucket::new(Default::default()));
}

#[update]
#[allow(dead_code)]
async fn set_bigmap_idx_can(bigmap_idx_can: CanisterId) -> Result<(), String> {
    let mut bm_data = match (&*BM_DATA).lock() {
        Ok(v) => v,
        Err(err) => return Err(format!("Failed to lock the BD Bucket, due to err: {}", err)),
    };

    bm_data.set_index_canister(bigmap_idx_can);
    Ok(())
}

#[update]
#[allow(dead_code)]
async fn put(key: Key, value: Val) {
    let mut bm_data = match (&*BM_DATA).lock() {
        Ok(v) => v,
        Err(err) => {
            println!("Failed to lock the BD Bucket, due to err: {}", err);
            return;
        }
    };

    println!("BigMap data: insert key {}", String::from_utf8_lossy(&key));
    bm_data.insert(key, value);
}

#[update]
#[allow(dead_code)]
async fn reset() {
    let bm_data = match (&*BM_DATA).lock() {
        Ok(v) => v,
        Err(err) => {
            println!("Failed to lock the BD Bucket, due to err: {}", err);
            return;
        }
    };

    println!("BigMap data: reset FIXME: implement");
}

#[query]
#[allow(dead_code)]
async fn get(key: Key) -> Option<Val> {
    let bm_data = match (&*BM_DATA).lock() {
        Ok(v) => v,
        Err(err) => {
            println!("Failed to lock the BD Bucket, due to err: {}", err);
            return None;
        }
    };

    println!("BigMap data: get key {}", String::from_utf8_lossy(&key));
    bm_data.get(key).ok()
}

#[query]
#[allow(dead_code)]
async fn holds_key(key: Key) -> Result<bool, String> {
    let bm_data = match (&*BM_DATA).lock() {
        Ok(v) => v,
        Err(err) => return Err(format!("Failed to lock the BD Bucket, due to err: {}", err)),
    };

    Ok(bm_data.holds_key(&key))
}

#[query]
#[allow(dead_code)]
async fn used_bytes(_: ()) -> Result<usize, String> {
    let bm_data = match (&*BM_DATA).lock() {
        Ok(v) => v,
        Err(err) => return Err(format!("Failed to lock the BD Bucket, due to err: {}", err)),
    };

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

    println!("BigMap data: pop_entries_for_canister_id {}", can_id);

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

fn main() {}
