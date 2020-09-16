use super::{calc_sha256, CanisterId, DataBucket};
use crate::hashring_sha256::{SHA256_DIGEST_MAX, SHA256_DIGEST_MIN};

#[actix_rt::test]
async fn bm_data_put_get() {
    // Insert key&value pairs and then get the value, and verify the correctness
    let mut d = DataBucket::new(CanisterId::from(42));
    d.set_range(&SHA256_DIGEST_MIN, &SHA256_DIGEST_MAX);
    for i in 0..100 as u8 {
        let key = format!("key-{}", i).into_bytes();
        let value = vec![i; 200_000];

        d.put(&key, &value, false).expect("DataBucket put failed");
        assert!(d.used_bytes() >= i as usize * 200_000);
    }

    for i in 0..100 as u8 {
        let key = format!("key-{}", i).into_bytes();
        let value = vec![i; 200_000];

        assert_eq!(*d.get(key).unwrap(), value);
    }
}

#[actix_rt::test]
async fn bm_data_append_get() {
    // Insert key&value pairs and then get the value, and verify the correctness
    let mut d = DataBucket::new(CanisterId::from(42));
    d.set_range(&SHA256_DIGEST_MIN, &SHA256_DIGEST_MAX);
    for i in 0..100 as u8 {
        let key = b"key-0".to_vec();
        let value = vec![i; 200_000];

        d.put(&key, &value, true).expect("DataBucket append failed");
        assert!(d.used_bytes() >= i as usize * 200_000);
    }

    let key = b"key-0".to_vec();
    let mut value = Vec::new();

    for i in 0..100 as u8 {
        value.extend(vec![i; 200_000]);
    }

    assert_eq!(*d.get(key).unwrap(), value);
}

#[test]
fn bm_data_hash_range_get() {
    // Insert key&value pairs and then get the value, and verify the correctness
    let mut d = DataBucket::new(CanisterId::from(42));
    d.set_range(&SHA256_DIGEST_MIN, &SHA256_DIGEST_MAX);

    let mut key_hashes = Vec::new();

    for i in 0..100 as u8 {
        let key = format!("key-{}", i).into_bytes();
        let value = vec![i; 200_000];

        key_hashes.push(calc_sha256(&key));
        d.put(&key, &value, false).expect("DataBucket put failed");
    }

    key_hashes.sort();
    let d_key_hashes = d.get_key_hash_range().unwrap();
    assert_eq!(key_hashes[0], d_key_hashes.0);
    assert_eq!(key_hashes[key_hashes.len() - 1], d_key_hashes.1);
}
