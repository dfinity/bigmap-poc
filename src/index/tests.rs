use crate::data::DataBucket;
use crate::index::BigmapIdx;
use crate::{CanisterId, Key};
use indexmap::IndexMap;
use std::sync::Arc;
// use std::time::Instant;

#[test]
fn bigmap_insert_get() {
    // Insert key&value pairs and then get the value, and verify the correctness
    let mut bm_idx = BigmapIdx::new();
    let mut db_map = Arc::new(IndexMap::new());

    for i in 0..11 {
        let can_id = CanisterId::from(i);
        Arc::get_mut(&mut db_map)
            .expect("db_map unwrap failed")
            .insert(can_id.clone(), DataBucket::new(can_id));
    }
    bm_idx
        .add_canisters(db_map.keys().map(|v| v.clone()).collect())
        .unwrap();

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

    let xcq_used_bytes = |can_id: CanisterId| {
        let can_data = db_map.borrow().get(&can_id).unwrap();
        can_data.used_bytes()
    };
    bm_idx.set_fn_xcq_used_bytes(Box::new(xcq_used_bytes));

    let xcq_holds_key_fn = move |can_id: CanisterId, key: &Key| {
        let can_data = db_map.get(&can_id).unwrap();
        can_data.holds_key(key)
    };
    bm_idx.set_fn_xcq_holds_key(Box::new(xcq_holds_key_fn));

    bm_idx.rebalance().unwrap();

    for i in 0..1001 {
        let key = format!("key-{}", i).into_bytes();
        let value = vec![(i % 256) as u8; 200_000];

        let can_data_id = bm_idx.lookup(&key).unwrap();
        assert_eq!(can_data_id.len(), 1);
        let can_data = db_map.get_mut(&can_data_id[0]).unwrap();

        can_data.insert(key, value);
        assert!(can_data.used_bytes() > 0);
    }

    bm_idx.rebalance().unwrap();

    for i in 0..1001 {
        let key = format!("key-{}", i).into_bytes();
        let value = vec![(i % 256) as u8; 200_000];

        let can_data_id = bm_idx.lookup(&key).unwrap();
        assert_eq!(can_data_id.len(), 1);
        let can_data = db_map.get(&can_data_id[0]).unwrap();

        assert_eq!(can_data.get(key).unwrap(), value);
    }
}

#[test]
fn bigmap_insert_rebalance_get() {
    // Insert key&value pairs and then get the value, and verify the correctness
    let mut bm_idx = BigmapIdx::new();
    let mut db_map = IndexMap::new();
    let num_data_can_initial = 100;
    let num_data_can_added = 1;

    for i in 0..num_data_can_initial {
        let can_id = CanisterId::from(i);
        db_map.insert(can_id.clone(), DataBucket::new(can_id));
    }

    bm_idx
        .add_canisters(db_map.keys().map(|v| v.clone()).collect())
        .unwrap();

    for i in 0..10000 {
        let key = format!("key-{}", i).into_bytes();
        let value = vec![(i % 256) as u8; 200_000];

        let can_data_id = bm_idx.lookup(&key).unwrap();
        assert_eq!(can_data_id.len(), 1);
        let can_data = db_map.get_mut(&can_data_id[0]).unwrap();

        can_data.insert(key, value);
        assert!(can_data.used_bytes() > 0);
    }

    for i in 0..num_data_can_added {
        let can_id = CanisterId::from(i);
        db_map.insert(can_id.clone(), DataBucket::new(can_id));
    }

    bm_idx
        .add_canisters(db_map.keys().map(|v| v.clone()).collect())
        .unwrap();

    for i in 0..10000 {
        let key = format!("key-{}", i).into_bytes();
        let value = vec![(i % 256) as u8; 200_000];

        let can_data_id = bm_idx.lookup(&key).unwrap();
        assert_eq!(can_data_id.len(), 1);
        let can_data = db_map.get(&can_data_id[0]).unwrap();

        assert_eq!(can_data.get(key).unwrap(), value);
    }
}
