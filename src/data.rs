use crate::{calc_sha256, sha256_digest_from_vec, CanisterId, Key, Sha256Digest, Sha2Vec, Val};
#[cfg(target_arch = "wasm32")]
use ic_cdk::println;
use std::collections::BTreeMap;
// use std::hash::{BuildHasherDefault, Hash, Hasher};
// use wyhash::WyHash;

// pub type DetHashMap<K, V> = HashMap<K, V, BuildHasherDefault<WyHash>>;

#[derive(Clone, Debug, Default)]
pub struct DataBucket {
    pub entries: BTreeMap<Sha256Digest, (Key, Val)>, // Can be DetHashMap
    range_start: Sha256Digest,                       // This DataBucket holds entries
    range_end: Sha256Digest,                         // in [range_start..range_end]
    used_bytes: usize,
    bytes_to_send: usize,
    id: CanisterId,
}

#[allow(dead_code)]
impl DataBucket {
    pub fn new(id: CanisterId) -> Self {
        // println!("BigMap Data {}: new", id);
        Self {
            id,
            ..Default::default()
        }
    }

    pub fn set_range(&mut self, range_start: &Sha256Digest, range_end: &Sha256Digest) {
        println!(
            "BigMap Data: set_range {} .. {}",
            hex::encode(range_start),
            hex::encode(range_end)
        );
        self.range_start = range_start.clone();
        self.range_end = range_end.clone();
    }

    pub fn is_in_range(&self, key_sha2: &Sha256Digest) -> bool {
        key_sha2 >= &self.range_start && key_sha2 < &self.range_end
    }

    pub fn put(&mut self, key: &Key, value: &Val, append: bool) -> Result<u64, String> {
        // println!("BigMap Data: put {}", String::from_utf8_lossy(&key));
        let key_sha2 = calc_sha256(&key);
        if !self.is_in_range(&key_sha2) {
            return Err(format!(
                "Provided key {} with sha256 {} is not in the range assigned to this DataBucket",
                String::from_utf8_lossy(&key),
                hex::encode(&key_sha2)
            ));
        }

        self.used_bytes += key.len();
        self.used_bytes += value.len();
        self.used_bytes += 32; // for the Sha256 of the key (=32 bytes)
        let value_len;

        if append {
            match &self.entries.get_key_value(&key_sha2) {
                Some((_, (_, val_old))) => {
                    let value_new = [&val_old[..], &value[..]].concat();
                    value_len = value_new.len();
                    self.entries.insert(key_sha2, (key.clone(), value_new));
                }
                None => {
                    value_len = value.len();
                    self.entries.insert(key_sha2, (key.clone(), value.clone()));
                }
            }
        } else {
            if let Some((_, (k, v))) = self.entries.get_key_value(&key_sha2) {
                // previous value is getting overwritten, update the accounting
                self.used_bytes -= k.len() + v.len() + 32;
            }
            value_len = value.len();
            self.entries.insert(key_sha2, (key.clone(), value.clone()));
        }
        Ok(value_len as u64)
    }

    pub fn batch_put(&mut self, batch: &Vec<(Key, Val)>) -> u64 {
        let mut result = 0;

        for (key, value) in batch {
            match self.put(key, value, false) {
                Ok(_) => result += 1,
                Err(err) => {
                    let key_str = String::from_utf8_lossy(&key);
                    println!("BigMap Data: put key {} error: {}", key_str, err);
                }
            }
        }
        result
    }

    pub fn delete(&mut self, key: Key) -> Result<u64, String> {
        let key_sha2 = calc_sha256(&key);
        if !self.is_in_range(&key_sha2) {
            return Err("Provided key is not in the range assigned to this DataBucket".to_string());
        }

        Ok(match &self.entries.remove(&key_sha2) {
            Some((_, value)) => {
                let value_bytes = value.len();
                let bytes_freed = key.len() + value.len() + 32;
                self.used_bytes = self.used_bytes.saturating_sub(bytes_freed);
                value_bytes as u64
            }
            None => 0,
        })
    }

    pub fn get_relocation_batch(&self, batch_limit_bytes: u64) -> Vec<(Sha2Vec, Key, Val)> {
        let mut batch = Vec::new();
        let mut batch_size_bytes = 0;

        for (key_sha2, (key, value)) in self.entries.iter() {
            if !self.is_in_range(key_sha2) {
                if batch_size_bytes + (key.len() + value.len()) as u64 >= batch_limit_bytes {
                    break;
                }
                batch.push((key_sha2.to_vec(), key.clone(), value.clone()));
                batch_size_bytes += (key.len() + value.len()) as u64;
            }
        }

        batch
    }

    pub fn put_relocation_batch(&mut self, batch: &Vec<(Sha2Vec, Key, Val)>) -> u64 {
        let mut put_count = 0;

        for (key_sha2, key, value) in batch.iter() {
            let key_sha2 = sha256_digest_from_vec(key_sha2);
            if self.is_in_range(&key_sha2) {
                self.used_bytes += key.len();
                self.used_bytes += value.len();
                self.used_bytes += 32; // for the Sha256 of the key (=32 bytes)
                self.entries.insert(key_sha2, (key.clone(), value.clone()));
                put_count += 1;
            } else {
                println!(
                    "BigMap Data: key is not in the assigned data bucket range {}",
                    String::from_utf8_lossy(&key)
                );
            }
        }

        put_count
    }

    pub fn delete_entries(&mut self, keys_sha2: &Vec<Vec<u8>>) {
        for key_sha2 in keys_sha2 {
            let key_sha2 = sha256_digest_from_vec(key_sha2);
            match self.entries.remove(&key_sha2) {
                Some((key, value)) => {
                    self.used_bytes -= key.len();
                    self.used_bytes -= value.len();
                    self.used_bytes -= 32; // for the Sha256 of the key (=32 bytes)
                }
                None => {}
            }
        }
    }

    pub fn get(&self, key: Key) -> Result<&Val, String> {
        // println!(
        //     "BigMap Data: get {}",
        //     String::from_utf8_lossy(&key)
        // );
        let key_sha2 = calc_sha256(&key);
        match self.entries.get(&key_sha2) {
            Some((_, v)) => Ok(v),
            None => Err("Entry not found".to_string()),
        }
    }

    pub fn list(&self, key_prefix: &Key) -> Vec<Key> {
        let mut result = Vec::new();

        for (key, _) in self.entries.values() {
            if key.len() >= key_prefix.len() && &key[0..key_prefix.len()] == key_prefix.as_slice() {
                result.push(key.clone());
                if result.len() > 10000 {
                    // Safety brake, don't return too many entries
                    break;
                }
            }
        }

        result
    }

    pub fn holds_key(&self, key: &Key) -> bool {
        let key_sha2 = calc_sha256(&key);
        self.entries.get(&key_sha2).is_some()
    }

    pub fn used_bytes(&self) -> usize {
        // We use DataBucket with BTreeMap, and the size is precalculated
        // For DataBucket configuration with HashMap, usage can be calculated as:
        // (key_size + value_size + 8) bytes * 1.1
        // https://github.com/servo/servo/issues/6908
        self.used_bytes
    }

    pub fn canister_id(&self) -> CanisterId {
        self.id.clone()
    }

    pub fn set_canister_id(&mut self, can_id: CanisterId) {
        self.id = can_id
    }

    pub fn get_key_hash_range(&self) -> Option<(Sha256Digest, Sha256Digest)> {
        match (self.entries.keys().min(), self.entries.keys().max()) {
            (Some(min), Some(max)) => Some((*min, *max)),
            _ => None,
        }
    }

    // Returns a randomly generated and unused key that matches this data canister
    pub fn get_random_key(&self, seed: Option<String>) -> String {
        // // Once replica & dfx both support raw_rand, we can try seeding from raw_rand
        // let rand_key = match subnet_raw_rand().await {
        //     Ok(result) => result,
        //     Err(err) => {
        //         println!("Error invoking raw_rand: {}", err);
        //         Vec::new()
        //     }
        // };

        let mut rand_key = match seed {
            Some(seed) => hex::encode(calc_sha256(seed)),
            None => {
                let time_bytes = ic_cdk::time().to_be_bytes();
                hex::encode(calc_sha256(&time_bytes.to_vec()))
            }
        };

        for _i in 0..100 {
            // Only try this a limited number of times
            let rand_key_hash = calc_sha256(&Vec::from(rand_key.as_bytes()));
            if self.is_in_range(&rand_key_hash) && !self.entries.contains_key(&rand_key_hash) {
                // if _i > 0 {
                //     println!(
                //         "get_random_key: {} ==> hash {}, after {} attempts",
                //         &rand_key,
                //         hex::encode(&rand_key_hash),
                //         _i
                //     );
                // }
                return rand_key;
            }
            rand_key = hex::encode(rand_key_hash);
        }

        println!("get_random_key: failed to find an unused key in the range");
        "".to_string()
    }

    // Generates and inserts num_entries into the data bucket, each with entry_size_bytes
    // Returns a vector of the inserted keys
    pub fn seed_random_data(&mut self, num_entries: u32, entry_size_bytes: u32) -> Vec<String> {
        let mut key = self.get_random_key(None);
        let mut result = Vec::new();

        for _ in 0..num_entries {
            self.put(
                &Vec::from(key.as_bytes()),
                &vec![0u8; entry_size_bytes as usize],
                false,
            )
            .expect("Put should never fail");

            result.push(key.clone());
            key = self.get_random_key(Some(key));
            if key.is_empty() {
                println!("seed_random_data: failed to find a suitable random key");
                break;
            }
        }
        result
    }
}

#[cfg(test)]
mod tests;
