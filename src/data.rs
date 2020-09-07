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

    pub fn put(&mut self, key: Key, value: Val, append: bool) -> Result<u64, String> {
        // println!(
        //     "BigMap Data: put {}",
        //     String::from_utf8_lossy(&key)
        // );
        let key_sha2 = calc_sha256(&key);
        if !self.is_in_range(&key_sha2) {
            return Err("Provided key is not in the range assigned to this DataBucket".to_string());
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
                    self.entries.insert(key_sha2, (key, value_new));
                }
                None => {
                    value_len = value.len();
                    self.entries.insert(key_sha2, (key, value));
                }
            }
        } else {
            if let Some((_, (k, v))) = self.entries.get_key_value(&key_sha2) {
                // previous value is getting overwritten, update the accounting
                self.used_bytes -= k.len() + v.len() + 32;
            }
            value_len = value.len();
            self.entries.insert(key_sha2, (key, value));
        }
        Ok(value_len as u64)
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

    pub fn put_batch(&mut self, batch: &Vec<(Sha2Vec, Key, Val)>) -> u64 {
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
    pub fn get_random_key(&self) -> String {
        // // Once replica & dfx both support raw_rand, we can try to seed from this instead from time
        // let rand_key = match subnet_raw_rand().await {
        //     Ok(result) => result,
        //     Err(err) => {
        //         println!("Error invoking raw_rand: {}", err);
        //         Vec::new()
        //     }
        // };

        let time_bytes = ic_cdk::time().to_be_bytes();
        let mut rand_key = calc_sha256(&time_bytes.to_vec());
        for i in 0..100 {
            // Only try this a limited number of times
            let rand_key_hash = calc_sha256(&rand_key);
            if self.range_start <= rand_key
                && rand_key < self.range_end
                && !self.entries.contains_key(&rand_key_hash)
            {
                let result = hex::encode(rand_key);
                println!("get_random_key: found {} after {} attempts", result, i);
                return result;
            }
            rand_key = rand_key_hash;
        }
        println!("get_random_key: failed to find an unused key in the range");
        "".to_string()
    }

    // fn find_unused_key_from_here(&self, key_start: Sha256Digest) -> Option<Sha256Digest> {
    //     let biguint_start = sha256_digest_to_biguint(self.range_start);
    //     let biguint_end = sha256_digest_to_biguint(self.range_end);
    //     let mut biguint_key =
    //         &biguint_start + sha256_digest_to_biguint(key_start) % (&biguint_end - &biguint_start);
    //     for i in 0..100 {
    //         // Make a limited number of attempts to find an unused entry
    //         let key_sha256 = biguint_to_sha256_digest(&biguint_key);
    //         if !self.entries.contains_key(&key_sha256) {
    //             return Some(key_sha256);
    //         }
    //         if biguint_key >= biguint_end {
    //             break;
    //         } else {
    //             biguint_key += 1u32;
    //         }
    //     }
    //     None
    // }
}

#[cfg(test)]
mod tests;
