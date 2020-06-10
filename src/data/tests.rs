use super::{calc_sha256, CanisterId, DataBucket};

#[test]
fn bm_data_insert_get() {
    // Insert key&value pairs and then get the value, and verify the correctness
    let mut d = DataBucket::new(CanisterId::from(42));
    for i in 0..100 as u8 {
        let key = format!("key-{}", i).into_bytes();
        let value = vec![i; 200_000];

        d.insert(key, value);
        assert!(d.used_bytes() >= i as usize * 200_000);
    }

    for i in 0..100 as u8 {
        let key = format!("key-{}", i).into_bytes();
        let value = vec![i; 200_000];

        assert_eq!(d.get(key).unwrap(), value);
    }
}

#[test]
fn bm_data_hash_range_get() {
    // Insert key&value pairs and then get the value, and verify the correctness
    let mut d = DataBucket::new(CanisterId::from(42));

    let mut key_hashes = Vec::new();

    for i in 0..100 as u8 {
        let key = format!("key-{}", i).into_bytes();
        let value = vec![i; 200_000];

        key_hashes.push(calc_sha256(&key));
        d.insert(key, value);
    }

    key_hashes.sort();
    let d_key_hashes = d.get_key_hash_range().unwrap();
    assert_eq!(key_hashes[0], d_key_hashes.0);
    assert_eq!(key_hashes[key_hashes.len() - 1], d_key_hashes.1);
}
