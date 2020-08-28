#[cfg(not(target_arch = "wasm32"))]
use crate::Sha2Vec;
use crate::{calc_sha256, hashring_sha256, CanisterId, Key, Sha256Digest, Val};
use bytesize::ByteSize;
#[cfg(target_arch = "wasm32")]
use ic_cdk::println;
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::BuildHasherDefault;
use wyhash::WyHash;

// CanisterPtr allows us to have u64 instead of a full CanisterId
// in various parts of the BigMap Index
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct CanisterPtr(u32);
pub type DetHashSet<K> = HashSet<K, BuildHasherDefault<WyHash>>;
pub type DetHashMap<K, V> = HashMap<K, V, BuildHasherDefault<WyHash>>;

type HashRingRange = (Sha256Digest, Sha256Digest);

// Testing types
#[cfg(not(target_arch = "wasm32"))]
type FnPtrUsedBytes = Box<dyn Fn(CanisterId) -> usize>;
#[cfg(not(target_arch = "wasm32"))]
type FnPtrHoldsKey = Box<dyn Fn(CanisterId, &Key) -> bool>;
#[cfg(not(target_arch = "wasm32"))]
type FnPtrSetRange = Box<dyn Fn(CanisterId, Sha256Digest, Sha256Digest)>;
#[cfg(not(target_arch = "wasm32"))]
type FnPtrGetRelocationBatch = Box<dyn Fn(CanisterId, u64) -> Vec<(Sha2Vec, Key, Val)>>;
#[cfg(not(target_arch = "wasm32"))]
type FnPtrPutBatch = Box<dyn Fn(CanisterId, &Vec<(Sha2Vec, Key, Val)>) -> u64>;
#[cfg(not(target_arch = "wasm32"))]
type FnPtrDeleteEntries = Box<dyn Fn(CanisterId, &Vec<Vec<u8>>)>;

#[derive(Default)]
pub struct BigmapIdx {
    idx: Vec<CanisterId>, // indirection for CanisterId, to avoid many copies of CanisterIds
    hash_ring: hashring_sha256::HashRing<CanisterPtr>,
    rebalance_queue: VecDeque<CanisterPtr>,
    now_rebalancing_src_dst: Option<(CanisterPtr, CanisterPtr)>,
    is_rebalancing: bool,
    batch_limit_bytes: u64,
    num_canisters_needed: u32,
    canister_available_queue: VecDeque<CanisterId>,
    used_bytes_threshold: u64,
    used_bytes_total: u64,
    id: CanisterId,
    // Testing functions
    #[cfg(not(target_arch = "wasm32"))]
    fn_ptr_used_bytes: Option<FnPtrUsedBytes>,
    #[cfg(not(target_arch = "wasm32"))]
    fn_ptr_holds_key: Option<Box<dyn Fn(CanisterId, &Key) -> bool>>,
    #[cfg(not(target_arch = "wasm32"))]
    fn_ptr_set_range: Option<FnPtrSetRange>,
    #[cfg(not(target_arch = "wasm32"))]
    fn_ptr_get_relocation_batch: Option<FnPtrGetRelocationBatch>,
    #[cfg(not(target_arch = "wasm32"))]
    fn_ptr_put_batch: Option<FnPtrPutBatch>,
    #[cfg(not(target_arch = "wasm32"))]
    fn_ptr_delete_entries: Option<FnPtrDeleteEntries>,
}

#[allow(dead_code)]
impl BigmapIdx {
    pub fn new() -> Self {
        Self {
            num_canisters_needed: 1,
            used_bytes_threshold: 512 * 1024 * 1024,
            batch_limit_bytes: 512 * 1024,
            ..Default::default()
        }
    }

    pub fn canisters_needed(&self) -> u32 {
        // TODO: adjust this number as BigMap grows
        self.num_canisters_needed
    }

    fn can_ptr_to_canister_id(&self, can_ptr: &CanisterPtr) -> CanisterId {
        self.idx[can_ptr.0 as usize].clone()
    }

    // TODO: implement
    pub fn add_data_canister_wasm_binary(&mut self, _wasm_binary: &[u8]) {
        unimplemented!("add_data_canister_wasm_binary");
    }

    // TODO: Convert to the proper canister creation
    pub async fn add_canisters(&mut self, can_ids: Vec<CanisterId>) {
        // let mut new_can_util_vec = Vec::new();

        for can_id in can_ids {
            println!(
                "BigMap Index {}: Created Data CanisterId {}",
                self.id, can_id
            );

            // Add all canisters to the available queue
            self.canister_available_queue.push_back(can_id);

            if self.num_canisters_needed > 0 {
                self.num_canisters_needed -= 1;
            }
        }

        // No Data canister in BigMap yet, take one from the available queue
        if self.hash_ring.is_empty() && !self.canister_available_queue.is_empty() {
            let can_id = self.canister_available_queue.pop_front().unwrap();

            println!(
                "BigMap Index {}: Activating Data CanisterId {}",
                self.id, can_id
            );

            let range = self.hash_ring_add_canister_id(&can_id);
            self.update_dcan_set_range(&can_id, range.0, range.1).await
        };
    }

    // Returns the data bucket canister which should hold the key
    // If multiple canisters can hold the data due to rebalancing, we will
    // query all candidates and return the correct CanisterId
    pub async fn lookup_get(&self, key: &Key) -> Option<CanisterId> {
        let key_sha256 = calc_sha256(key);
        let (_, can_ptr) = match self.hash_ring.get_idx_node_for_key(&key_sha256) {
            Some(v) => v,
            None => return None,
        };
        // println!("BigMap Index {}: lookup_get @key {}", self.id, String::from_utf8_lossy(key));

        let can_id = self.can_ptr_to_canister_id(can_ptr);
        if self.query_dcan_holds_key(&can_id, key).await {
            return Some(can_id);
        }

        if let Some(can_src_dst_ptr) = self.now_rebalancing_src_dst {
            let (rebalance_src_ptr, rebalance_dst_ptr) = can_src_dst_ptr;
            if can_ptr == &rebalance_src_ptr {
                // The destination canister doesn't have the key but it's currently rebalancing.
                // The key may not have been moved yet from the source canister

                let can_id = self.can_ptr_to_canister_id(&rebalance_dst_ptr);
                if self.query_dcan_holds_key(&can_id, key).await {
                    println!(
                        "BigMap Index {}: lookup_get @key {} from a relocation destination {}",
                        self.id,
                        String::from_utf8_lossy(key),
                        can_id
                    );
                    return Some(can_id);
                }
            }
        }

        None
    }

    // Find the data bucket canister into which the object with the provided key should go
    pub fn lookup_put(&self, key: &Key) -> Option<CanisterId> {
        let key_sha256 = calc_sha256(key);
        let (_, ring_node) = match self.hash_ring.get_idx_node_for_key(&key_sha256) {
            Some(v) => v,
            None => return None,
        };

        // println!("BigMap Index {}: lookup_put @key {}", self.id, String::from_utf8_lossy(key));
        Some(self.can_ptr_to_canister_id(ring_node))
    }

    pub async fn maintenance(&mut self) -> Result<u8, String> {
        // println!(
        //     "BigMap Index {}: CanisterIds with pending rebalance {:?}",
        //     self.id,
        //     self.rebalance_queue
        //         .iter()
        //         .map(|can_ptr| self.can_ptr_to_canister_id(can_ptr))
        //         .collect::<Vec<CanisterId>>()
        // );
        if self.is_rebalancing {
            return Ok(10);
        }
        self.is_rebalancing = true;
        let result;

        match self.rebalance_queue.front() {
            Some(src_canister_ptr) => {
                // Some canister is ready to be rebalanced. We'll do these steps:
                // - Create destination canister, to which half of the data from the source canister will go
                // - Move batches of objects from source canister to the destination canister
                if self.now_rebalancing_src_dst.is_none() {
                    // We're just starting to rebalance, create the destination canister
                    let src_canister_ptr = src_canister_ptr.clone();
                    let src_canister = self.can_ptr_to_canister_id(&src_canister_ptr);
                    let dst_canister = self
                        .create_data_bucket_canister()
                        .await
                        .expect("create_data_bucket_canister failed");

                    let (dst_canister_ptr, range_dst, range_src) =
                        self.hash_ring_add_before_this(&src_canister_ptr, &dst_canister.clone());

                    // Now the new canister has been created and added to the hash ring

                    // Update the range covered by Source and Destination canister
                    self.update_dcan_set_range(&src_canister, range_src.0, range_src.1)
                        .await;
                    self.update_dcan_set_range(&dst_canister, range_dst.0, range_dst.1)
                        .await;

                    // Remember the canisters we're currently rebalancing, for future invocations
                    self.now_rebalancing_src_dst = Some((src_canister_ptr, dst_canister_ptr))

                    // And we're done adding the canister, now we only need to move the data
                }

                let (rebalance_src_ptr, rebalance_dst_ptr) = self.now_rebalancing_src_dst.unwrap();
                let src_canister = self.can_ptr_to_canister_id(&rebalance_src_ptr);
                let dst_canister = self.can_ptr_to_canister_id(&rebalance_dst_ptr);

                let batch = self
                    .update_dcan_get_relocation_batch(&src_canister, self.batch_limit_bytes)
                    .await;
                if !batch.is_empty() {
                    let put_count = self.update_dcan_put_batch(&dst_canister, &batch).await;
                    if batch.len() as u64 != put_count {
                        println!(
                            "BigMap Index {}: Not all elements were moved from {} to {}",
                            self.id, src_canister, dst_canister
                        )
                    }
                    let batch_sha2 = batch.iter().map(|e| e.0.clone()).collect();

                    self.update_dcan_delete_entries(&src_canister, &batch_sha2)
                        .await;
                    result = Ok(1);
                } else {
                    println!(
                        "BigMap Index {}: All pending elements moved from {} to {}",
                        self.id, src_canister, dst_canister
                    );
                    self.now_rebalancing_src_dst = None;
                    self.rebalance_queue.pop_front();
                    if self.rebalance_queue.is_empty() {
                        self.query_all_canisters_for_used_bytes_and_enqueue_rebalance()
                            .await;
                        if self.rebalance_queue.is_empty() {
                            result = Ok(0)
                        } else {
                            result = Ok(1)
                        }
                    } else {
                        result = Ok(1)
                    }
                }
            }
            None => {
                self.query_all_canisters_for_used_bytes_and_enqueue_rebalance()
                    .await;
                if self.rebalance_queue.is_empty() {
                    result = Ok(0); // Everything is balanced
                } else {
                    // We need to rebalance data, call me again ASAP
                    result = Ok(1);
                }
            }
        }
        self.is_rebalancing = false;
        result
    }

    async fn query_all_canisters_for_used_bytes_and_enqueue_rebalance(&mut self) {
        self.used_bytes_total = 0;

        for (i, can_id) in self.idx.iter().enumerate() {
            let can_ptr = CanisterPtr { 0: i as u32 };
            let used_bytes = self.query_dcan_used_bytes(can_id).await as u64;
            self.print_canister_utilization(can_id, used_bytes);
            if used_bytes > self.used_bytes_threshold {
                self.rebalance_queue.push_back(can_ptr);
            }
            self.used_bytes_total += used_bytes;
        }
        println!("Total capacity used {}", ByteSize(self.used_bytes_total));
    }

    fn hash_ring_add_before_this(
        &mut self,
        can_ptr: &CanisterPtr,
        can_id_new: &CanisterId,
    ) -> (CanisterPtr, HashRingRange, HashRingRange) {
        let can_ptr_new = CanisterPtr {
            0: self.idx.len() as u32,
        };
        // Query the Hash Ring for the information on the provided (existing) Canister
        let (hr_i, hr_key, _) = self.hash_ring.get_idx_key_node_for_node(can_ptr).unwrap();
        // Get the value of the previous key in the hash ring
        let hr_key_prev = if hr_i == 0 {
            // There is no previous key
            *hashring_sha256::SHA256_DIGEST_MIN
        } else {
            self.hash_ring.get_prev_key_node_at_idx(hr_i).unwrap().0
        };
        // Now calculate the middle point between hr_key_prev and hr_key
        let hr_key_new = hashring_sha256::sha256_range_half(&hr_key_prev, &hr_key);
        self.hash_ring.add_with_key(&hr_key_new, can_ptr_new);

        self.idx.push(can_id_new.clone());

        // The entries in the hash ring are now updated
        let hr_idx_new_canister = hr_i; // == self.hash_ring.get_idx_key_node_for_node(can_ptr_new).unwrap().0;
        let hr_idx_old_canister = hr_i + 1; // == self.hash_ring.get_idx_key_node_for_node(can_ptr).unwrap().0;
        (
            can_ptr_new,
            self.hash_ring.get_key_range_for_idx(hr_idx_new_canister),
            self.hash_ring.get_key_range_for_idx(hr_idx_old_canister),
        )
    }

    fn hash_ring_add_canister_id(&mut self, can_id: &CanisterId) -> HashRingRange {
        let ptr_new = CanisterPtr {
            0: self.idx.len() as u32,
        };
        self.hash_ring.add(ptr_new.clone());

        self.idx.push(can_id.clone());

        let hr_idx = self
            .hash_ring
            .get_idx_key_node_for_node(&ptr_new)
            .unwrap()
            .0;
        self.hash_ring.get_key_range_for_idx(hr_idx)
    }

    pub fn set_used_bytes_threshold(&mut self, used_bytes_threshold: u64) {
        self.used_bytes_threshold = used_bytes_threshold;
    }

    pub fn set_canister_id(&mut self, can_id: CanisterId) {
        self.id = can_id
    }

    pub fn canister_id(&self) -> CanisterId {
        self.id.clone()
    }

    async fn create_data_bucket_canister(&mut self) -> Result<CanisterId, String> {
        match self.canister_available_queue.pop_front() {
            Some(can_id) => Ok(can_id),
            None => unimplemented!("create_data_bucket_canister"),
        }
    }

    fn print_canister_utilization(&self, can_id: &CanisterId, used_bytes: u64) {
        println!(
            "CanisterId {} used {}",
            can_id.clone(),
            ByteSize(used_bytes)
        );
    }
}

#[cfg(target_arch = "wasm32")]
impl BigmapIdx {
    async fn query_dcan_used_bytes(&self, can_id: &CanisterId) -> usize {
        ic_cdk::call(can_id.clone().0.into(), "used_bytes", Some(()))
            .await
            .expect("used_bytes call failed")
    }

    async fn query_dcan_holds_key(&self, can_id: &CanisterId, key: &Key) -> bool {
        ic_cdk::call(can_id.clone().0.into(), "holds_key", Some(key))
            .await
            .expect("holds_key call failed")
    }

    async fn update_dcan_set_range(
        &self,
        can_id: &CanisterId,
        range_start: Sha256Digest,
        range_end: Sha256Digest,
    ) {
        ic_cdk::call(
            can_id.clone().0.into(),
            "set_range",
            Some((range_start.to_vec(), range_end.to_vec())),
        )
        .await
        .expect("set_range call failed")
    }

    async fn update_dcan_get_relocation_batch(
        &self,
        can_id: &CanisterId,
        batch_size_bytes: u64,
    ) -> Vec<(Key, Val)> {
        ic_cdk::call(
            can_id.clone().0.into(),
            "get_relocation_batch",
            Some(batch_size_bytes),
        )
        .await
        .expect("get_relocation_batch call failed")
    }

    async fn update_dcan_put_batch(&self, can_id: &CanisterId, batch: &Vec<(Key, Val)>) -> u64 {
        ic_cdk::call(can_id.clone().0.into(), "put_batch", Some(batch))
            .await
            .expect("put_batch call failed")
    }

    async fn update_dcan_delete_entries(&self, can_id: &CanisterId, keys_sha2: &Vec<Vec<u8>>) {
        ic_cdk::call(can_id.clone().0.into(), "delete_entries", Some(keys_sha2))
            .await
            .expect("delete_entries call failed")
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl BigmapIdx {
    //////////////////////////////////////////////////////////////////////////
    //
    // Testing helpers
    //
    //////////////////////////////////////////////////////////////////////////

    pub fn set_fn_ptr_used_bytes(&mut self, fn_ptr: FnPtrUsedBytes) {
        self.fn_ptr_used_bytes = Some(fn_ptr);
    }

    pub fn set_fn_ptr_set_range(&mut self, fn_ptr: FnPtrSetRange) {
        self.fn_ptr_set_range = Some(fn_ptr);
    }

    pub fn set_fn_ptr_holds_key(&mut self, fn_ptr: FnPtrHoldsKey) {
        self.fn_ptr_holds_key = Some(fn_ptr);
    }

    pub fn set_fn_ptr_get_relocation_batch(&mut self, fn_ptr: FnPtrGetRelocationBatch) {
        self.fn_ptr_get_relocation_batch = Some(fn_ptr);
    }

    pub fn set_fn_ptr_put_batch(&mut self, fn_ptr: FnPtrPutBatch) {
        self.fn_ptr_put_batch = Some(fn_ptr);
    }

    pub fn set_fn_ptr_delete_entries(&mut self, fn_ptr: FnPtrDeleteEntries) {
        self.fn_ptr_delete_entries = Some(fn_ptr);
    }

    async fn query_dcan_used_bytes(&self, can_id: &CanisterId) -> usize {
        let fn_ptr = self
            .fn_ptr_used_bytes
            .as_ref()
            .expect("fn_ptr_used_bytes is not set");
        return fn_ptr(can_id.clone());
    }

    async fn query_dcan_holds_key(&self, can_id: &CanisterId, key: &Key) -> bool {
        let fn_ptr = self
            .fn_ptr_holds_key
            .as_ref()
            .expect("fn_ptr_used_bytes is not set");
        return fn_ptr(can_id.clone(), key);
    }

    async fn update_dcan_set_range(
        &self,
        can_id: &CanisterId,
        range_start: Sha256Digest,
        range_end: Sha256Digest,
    ) {
        let fn_ptr = self
            .fn_ptr_set_range
            .as_ref()
            .expect("fn_ptr_set_range is not set");
        return fn_ptr(can_id.clone(), range_start, range_end);
    }

    async fn update_dcan_get_relocation_batch(
        &self,
        can_id: &CanisterId,
        batch_limit_bytes: u64,
    ) -> Vec<(Sha2Vec, Key, Val)> {
        let fn_ptr = self
            .fn_ptr_get_relocation_batch
            .as_ref()
            .expect("fn_ptr_get_relocation_batch is not set");
        fn_ptr(can_id.clone(), batch_limit_bytes)
    }

    async fn update_dcan_put_batch(
        &self,
        can_id: &CanisterId,
        batch: &Vec<(Sha2Vec, Key, Val)>,
    ) -> u64 {
        let fn_ptr = self
            .fn_ptr_put_batch
            .as_ref()
            .expect("fn_ptr_put_batch is not set");
        fn_ptr(can_id.clone(), batch)
    }

    async fn update_dcan_delete_entries(&self, can_id: &CanisterId, keys_sha2: &Vec<Vec<u8>>) {
        let fn_ptr = self
            .fn_ptr_delete_entries
            .as_ref()
            .expect("fn_ptr_delete_entries is not set");
        return fn_ptr(can_id.clone(), keys_sha2);
    }
}

#[cfg(test)]
mod tests;
