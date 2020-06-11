use crate::data::DataBucket;
use crate::index::BigmapIdx;
use crate::{CanisterId, Key};
use indexmap::IndexMap;
use std::sync::{Arc, Mutex};
// use std::time::Instant;

#[test]
fn bigmap_put_get() {
    // Insert key&value pairs and then get the value, and verify the correctness
    let mut bm_idx = BigmapIdx::new();
    let db_map = Arc::new(Mutex::new(IndexMap::new()));

    for i in 0..11 {
        let can_id = CanisterId::from(i);
        db_map
            .lock()
            .unwrap()
            .insert(can_id.clone(), DataBucket::new(can_id));
    }
    bm_idx.add_canisters(db_map.lock().unwrap().keys().cloned().collect());

    let db_map_ref1 = db_map.clone();
    let xcq_used_bytes = move |can_id: CanisterId| {
        db_map_ref1
            .lock()
            .unwrap()
            .get(&can_id)
            .unwrap()
            .used_bytes()
    };
    bm_idx.set_fn_xcq_used_bytes(Box::new(xcq_used_bytes));

    let db_map_ref2 = db_map.clone();
    let xcq_holds_key_fn = move |can_id: CanisterId, key: &Key| {
        db_map_ref2
            .lock()
            .unwrap()
            .get(&can_id)
            .unwrap()
            .holds_key(key)
    };
    bm_idx.set_fn_xcq_holds_key(Box::new(xcq_holds_key_fn));

    bm_idx.rebalance().unwrap();

    for i in 0..1001 {
        let key = format!("key-{}", i).into_bytes();
        let value = vec![(i % 256) as u8; 200_000];

        let can_data_id = bm_idx.lookup_put(&key).unwrap();
        assert_ne!(can_data_id, Default::default());
        let d = db_map.lock();
        // println!("db_map: {:?}", d);
        d.unwrap().get_mut(&can_data_id).unwrap().put(key, value);
        assert!(
            db_map
                .lock()
                .unwrap()
                .get(&can_data_id)
                .unwrap()
                .used_bytes()
                > 0
        );
    }

    bm_idx.rebalance().unwrap();

    for i in 0..1001 {
        let key = format!("key-{}", i).into_bytes();
        let value = vec![(i % 256) as u8; 200_000];

        let can_data_id = bm_idx.lookup_get(&key).unwrap();
        assert_ne!(can_data_id, Default::default());
        let can_data = db_map.lock().unwrap();
        let can_data = can_data.get(&can_data_id).unwrap();

        assert_eq!(can_data.get(key).unwrap(), value);
    }
}

#[ignore]
#[test]
fn bigmap_put_rebalance_get() {
    // Insert key&value pairs and then get the value, and verify the correctness
    let mut bm_idx = BigmapIdx::new();
    let db_map = Arc::new(Mutex::new(IndexMap::new()));
    let num_data_can_initial = 100;
    let num_data_can_added = 1;

    for i in 0..num_data_can_initial {
        let can_id = CanisterId::from(i);
        db_map
            .lock()
            .unwrap()
            .insert(can_id.clone(), DataBucket::new(can_id));
    }

    let db_map_ref2 = db_map.clone();
    let xcq_holds_key_fn = move |can_id: CanisterId, key: &Key| {
        db_map_ref2
            .lock()
            .unwrap()
            .get(&can_id)
            .unwrap()
            .holds_key(key)
    };
    bm_idx.set_fn_xcq_holds_key(Box::new(xcq_holds_key_fn));

    bm_idx.add_canisters(db_map.lock().unwrap().keys().cloned().collect());

    for i in 0..10000 {
        let key = format!("key-{}", i).into_bytes();
        let value = vec![(i % 256) as u8; 200_000];

        let can_data_id = bm_idx.lookup_put(&key).unwrap();
        assert_ne!(can_data_id, Default::default());
        let mut can_data = db_map.lock().unwrap();
        let can_data = can_data.get_mut(&can_data_id).unwrap();

        can_data.put(key, value);
        assert!(can_data.used_bytes() > 0);
    }

    for i in 0..num_data_can_added {
        let can_id = CanisterId::from(i);
        db_map
            .lock()
            .unwrap()
            .insert(can_id.clone(), DataBucket::new(can_id));
    }

    bm_idx.add_canisters(db_map.lock().unwrap().keys().cloned().collect());

    /*
        let bigmap_maintenance = || {
        println!(">>>>> Big map maintenance starting");
        bm_idx.canisters_needed();
        let mut data_can_vec = Vec::new(); // create multiple data canisters at once
        println!("Big map add {} data canister(s)", data_can_vec.len());
        // update calls are expensive, send multiple data canisters in one go
        let res: Result<(), String> = bm_idx.update("add_data_buckets").json(data_can_vec);
        println!("Big map add data can DONE: {:?}", res);

        println!("Big map rebalancing");
        let res: Result<u8, String> = bm_idx.update("rebalance").json(());
        println!("Big map rebalance completed: {:?}", res);
        println!(">>>>> Big map maintenance completed in: {:?}",);
        };
    */

    for i in 0..10000 {
        let key = format!("key-{}", i).into_bytes();
        let value = vec![(i % 256) as u8; 200_000];

        let can_data_id = bm_idx.lookup_get(&key).unwrap();
        assert_ne!(can_data_id, Default::default());
        let can_data = db_map.lock().unwrap();
        let can_data = can_data.get(&can_data_id).unwrap();

        assert_eq!(can_data.get(key).unwrap(), value);
    }
}
