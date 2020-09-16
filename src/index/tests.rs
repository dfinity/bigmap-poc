use crate::data::DataBucket;
use crate::index::BigmapIdx;
use crate::{CanisterId, Key, Sha256Digest, Sha2Vec, Val};
use indexmap::IndexMap;
use std::collections::BTreeSet;
use std::sync::{Arc, RwLock};
// use std::time::Instant;

type DataBucketMap = Arc<RwLock<IndexMap<CanisterId, DataBucket>>>;

#[actix_rt::test]
async fn bigmap_put_get() {
    // Insert key&value pairs and then get the value, and verify the correctness

    let num_data_canisters_initial = 11;

    let (mut bm_idx, db_map) = alloc_bigmap_index_and_data(num_data_canisters_initial).await;

    bm_idx.maintenance().await;

    for i in 0..1001 {
        let key = format!("key-{}", i).into_bytes();
        let value = vec![(i % 256) as u8; 200_000];

        let can_data_id = bm_idx.lookup_put(&key).unwrap();
        assert_ne!(can_data_id, Default::default());
        db_map
            .write()
            .unwrap()
            .get_mut(&can_data_id)
            .unwrap()
            .put(&key, &value, false)
            .expect("DataBucket put failed");
        assert!(
            db_map
                .read()
                .unwrap()
                .get(&can_data_id)
                .unwrap()
                .used_bytes()
                > 0
        );
    }

    bm_idx.maintenance().await;

    for i in 0..1001 {
        let key = format!("key-{}", i).into_bytes();
        let value = vec![(i % 256) as u8; 200_000];

        let can_data_id = bm_idx.lookup_get(&key).await.unwrap();
        assert_ne!(can_data_id, Default::default());
        let can_data = db_map.read().unwrap();
        let can_data = can_data.get(&can_data_id).unwrap();

        assert_eq!(*can_data.get(key).unwrap(), value);
    }
}

#[actix_rt::test]
async fn bigmap_list() {
    // Create N canisters, write keys to them, then list keys and verify the list is as expected

    let num_data_canisters_initial = 20;

    let (mut bm_idx, db_map) = alloc_bigmap_index_and_data(num_data_canisters_initial).await;

    bm_idx.maintenance().await;

    let mut keys_expected_no_prefix = BTreeSet::new();
    let mut keys_expected_with_prefix = BTreeSet::new();
    let key_prefix = "key-1".to_string();
    for i in 0..1001 {
        let key = format!("key-{}", i);
        keys_expected_no_prefix.insert(key.clone());
        if key.starts_with(&key_prefix) {
            keys_expected_with_prefix.insert(key.clone());
        }

        let key = key.into_bytes();
        let value = vec![(i % 256) as u8; 20];

        let can_data_id = bm_idx.lookup_put(&key).unwrap();
        assert_ne!(can_data_id, Default::default());
        db_map
            .write()
            .unwrap()
            .get_mut(&can_data_id)
            .unwrap()
            .put(&key, &value, false)
            .expect("DataBucket put failed");
        assert!(
            db_map
                .read()
                .unwrap()
                .get(&can_data_id)
                .unwrap()
                .used_bytes()
                > 0
        );
    }

    bm_idx.set_used_bytes_threshold(5000);

    for _ in 0..5u32 {
        bm_idx.maintenance().await;
    }

    let list_keys = bm_idx.list(&Vec::new()).await;
    for (key, key_expected) in list_keys.iter().zip(keys_expected_no_prefix) {
        let key = String::from_utf8_lossy(key);
        assert_eq!(key, key_expected);
    }

    let list_keys = bm_idx.list(&key_prefix.into_bytes()).await;
    for (key, key_expected) in list_keys.iter().zip(keys_expected_with_prefix) {
        let key = String::from_utf8_lossy(key);
        assert_eq!(key, key_expected);
    }
}

#[actix_rt::test]
async fn bigmap_put_rebalance_get() {
    let num_data_canisters_initial = 10;
    let num_entries = 20000;

    let (mut bm_idx, db_map) = alloc_bigmap_index_and_data(num_data_canisters_initial).await;

    // Insert some elements into BigMap
    for i in 0..num_entries {
        let key = format!("key-{}", i).into_bytes();
        let value = vec![(i % 256) as u8; 200_000];

        let can_data_id = bm_idx.lookup_put(&key).unwrap();
        assert_ne!(can_data_id, Default::default());
        let mut can_data = db_map.write().unwrap();
        let can_data = can_data.get_mut(&can_data_id).unwrap();

        can_data
            .put(&key, &value, false)
            .expect("DataBucket put failed");
        assert!(can_data.used_bytes() > 0);
    }

    bm_idx.maintenance().await;

    // Check that all values are still retrievable from the BigMap
    for i in 0..num_entries {
        let key = format!("key-{}", i).into_bytes();
        let value = vec![(i % 256) as u8; 200_000];

        let can_data_id = bm_idx.lookup_get(&key).await.unwrap();
        assert_ne!(can_data_id, Default::default());
        let can_data = db_map.read().unwrap();
        let can_data = can_data.get(&can_data_id).unwrap();

        assert_eq!(*can_data.get(key).unwrap(), value);
    }
}

async fn alloc_bigmap_index_and_data(num_data_canisters: u64) -> (BigmapIdx, DataBucketMap) {
    // Insert key&value pairs and then get the value, and verify the correctness
    let mut bm_idx = BigmapIdx::new();

    // Note that due to the locks it's only possible to invoke one of these mocked functions at a time
    let db_map = Arc::new(RwLock::new(IndexMap::new()));

    for i in 0..num_data_canisters {
        let can_id = CanisterId::from(i);
        db_map
            .write()
            .unwrap()
            .insert(can_id.clone(), DataBucket::new(can_id));
    }

    let db_map_ref = db_map.clone();
    let fn_ptr_used_bytes = move |can_id: CanisterId| {
        db_map_ref
            .read()
            .unwrap()
            .get(&can_id)
            .unwrap()
            .used_bytes()
    };
    bm_idx.set_fn_ptr_used_bytes(Box::new(fn_ptr_used_bytes));

    let db_map_ref = db_map.clone();
    let fn_ptr_holds_key = move |can_id: CanisterId, key: &Key| {
        db_map_ref
            .read()
            .unwrap()
            .get(&can_id)
            .unwrap()
            .holds_key(key)
    };
    bm_idx.set_fn_ptr_holds_key(Box::new(fn_ptr_holds_key));

    let db_map_ref = db_map.clone();
    let fn_ptr_list = move |can_id: CanisterId, key_prefix: &Key| {
        db_map_ref
            .read()
            .unwrap()
            .get(&can_id)
            .unwrap()
            .list(key_prefix)
    };
    bm_idx.set_fn_ptr_list(Box::new(fn_ptr_list));

    let db_map_ref = db_map.clone();
    let fn_ptr_set_range =
        move |can_id: CanisterId, range_start: Sha256Digest, range_end: Sha256Digest| {
            db_map_ref
                .write()
                .unwrap()
                .get_mut(&can_id)
                .unwrap()
                .set_range(&range_start, &range_end)
        };
    bm_idx.set_fn_ptr_set_range(Box::new(fn_ptr_set_range));

    let db_map_ref = db_map.clone();
    let fn_ptr = move |can_id: CanisterId, batch_limit_bytes: u64| {
        db_map_ref
            .write()
            .unwrap()
            .get(&can_id)
            .unwrap()
            .get_relocation_batch(batch_limit_bytes)
    };
    bm_idx.set_fn_ptr_get_relocation_batch(Box::new(fn_ptr));

    let db_map_ref = db_map.clone();
    let fn_ptr = move |can_id: CanisterId, batch: &Vec<(Sha2Vec, Key, Val)>| {
        db_map_ref
            .write()
            .unwrap()
            .get_mut(&can_id)
            .unwrap()
            .put_relocation_batch(batch)
    };
    bm_idx.set_fn_ptr_put_relocation_batch(Box::new(fn_ptr));

    let db_map_ref = db_map.clone();
    let fn_ptr = move |can_id: CanisterId, keys_sha2: &Vec<Vec<u8>>| {
        db_map_ref
            .write()
            .unwrap()
            .get_mut(&can_id)
            .unwrap()
            .delete_entries(keys_sha2)
    };
    bm_idx.set_fn_ptr_delete_entries(Box::new(fn_ptr));

    let can_ids = db_map.write().unwrap().keys().cloned().collect();
    bm_idx.add_canisters(can_ids).await;

    (bm_idx, db_map)
}
