use crate::{calc_sha256, CanisterId, Key, Sha256Digest, Val};
#[cfg(target_arch = "wasm32")]
use ic_cdk::println;
use std::collections::BTreeMap;
// use std::hash::{BuildHasherDefault, Hash, Hasher};
// use wyhash::WyHash;

// pub type DetHashMap<K, V> = HashMap<K, V, BuildHasherDefault<WyHash>>;

#[derive(Clone, Debug, Default)]
pub struct DataBucket {
    pub index_canister: CanisterId,
    pub rebalance_from_data_can: Option<CanisterId>,
    pub rebalance_to_data_can: Option<CanisterId>,
    pub entries: BTreeMap<Sha256Digest, (Key, Val)>,
    used_bytes: usize,
    id: CanisterId,
}

#[allow(dead_code)]
impl DataBucket {
    pub fn new(id: CanisterId) -> Self {
        // println!("DataBucket new {}", id);
        Self {
            id,
            ..Default::default()
        }
    }

    pub fn set_index_canister(&mut self, idx_can_id: CanisterId) {
        self.index_canister = idx_can_id
    }

    pub fn put(&mut self, key: Key, value: Val) {
        println!(
            "DataBucket {} put {}",
            self.id,
            String::from_utf8_lossy(&key)
        );
        self.used_bytes += key.len();
        self.used_bytes += value.len();
        self.used_bytes += 32; // for the Sha256 of the key (=32 bytes)
        let key_sha2 = calc_sha256(&key);
        if let Some((_, (k, v))) = self.entries.get_key_value(&key_sha2) {
            // previous value is getting overwritten, update the accounting
            self.used_bytes -= k.len() + v.len() + 32;
        }
        self.entries.insert(key_sha2, (key, value));
    }

    pub fn get(&self, key: Key) -> Result<Val, String> {
        println!(
            "DataBucket {} get {}",
            self.id,
            String::from_utf8_lossy(&key)
        );
        let key_sha2 = calc_sha256(&key);
        match self.entries.get(&key_sha2) {
            Some((_, v)) => Ok(v.to_vec()),
            None => Err("Entry not found".to_string()),
        }
    }

    pub fn holds_key(&self, key: &Key) -> bool {
        let key_sha2 = calc_sha256(&key);
        self.entries.get(&key_sha2).is_some()
    }

    pub fn used_bytes(&self) -> usize {
        // Hash map usage: (key+val+8) bytes * 1.1
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

    pub fn set_rebalance_from_data_can(&mut self, can_id: Option<CanisterId>) {
        self.rebalance_from_data_can = can_id
    }

    pub fn set_rebalance_to_data_can(&mut self, can_id: Option<CanisterId>) {
        self.rebalance_to_data_can = can_id
    }

    pub fn get_rebalance_from_data_can(&self) -> Option<CanisterId> {
        self.rebalance_from_data_can.clone()
    }

    pub fn get_rebalance_to_data_can(&self) -> Option<CanisterId> {
        self.rebalance_to_data_can.clone()
    }
}

#[cfg(test)]
mod tests;
