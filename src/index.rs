use crate::{
    calc_sha256, hashring_sha256, subnet_create_new_canister, subnet_install_canister_code,
    CanisterId, Key, Sha256Digest, Sha2Vec, Val,
};
use bytesize::ByteSize;
#[cfg(target_arch = "wasm32")]
use ic_cdk::println;
use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
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
type FnPtrAddToSearchIndex = Box<dyn Fn(CanisterId, &Key, &String)>;
#[cfg(not(target_arch = "wasm32"))]
type FnPtrBatchAddToSearchIndex = Box<dyn Fn(CanisterId, &Vec<(Key, String)>) -> u64>;
#[cfg(not(target_arch = "wasm32"))]
type FnPtrSearchKeysByQuery = Box<dyn Fn(CanisterId, &String) -> Vec<Key>>;
#[cfg(not(target_arch = "wasm32"))]
type FnPtrRemoveFromSearchIndex = Box<dyn Fn(CanisterId, &Key)>;
#[cfg(not(target_arch = "wasm32"))]
type FnPtrList = Box<dyn Fn(CanisterId, &Key) -> Vec<Key>>;
#[cfg(not(target_arch = "wasm32"))]
type FnPtrUsedBytes = Box<dyn Fn(CanisterId) -> usize>;
#[cfg(not(target_arch = "wasm32"))]
type FnPtrHoldsKey = Box<dyn Fn(CanisterId, &Key) -> bool>;
#[cfg(not(target_arch = "wasm32"))]
type FnPtrSetRange = Box<dyn Fn(CanisterId, Sha256Digest, Sha256Digest)>;
#[cfg(not(target_arch = "wasm32"))]
type FnPtrGetRelocationBatch = Box<dyn Fn(CanisterId, u64) -> Vec<(Sha2Vec, Key, Val)>>;
#[cfg(not(target_arch = "wasm32"))]
type FnPtrPutRelocationBatch = Box<dyn Fn(CanisterId, &Vec<(Sha2Vec, Key, Val)>) -> u64>;
#[cfg(not(target_arch = "wasm32"))]
type FnPtrDeleteEntries = Box<dyn Fn(CanisterId, &Vec<Vec<u8>>)>;

#[derive(Default)]
pub struct BigmapIdx {
    idx: Vec<CanisterId>, // indirection for CanisterId, to avoid many copies of CanisterIds
    hash_ring: hashring_sha256::HashRing<CanisterPtr>,
    now_rebalancing_src_dst: Option<(CanisterPtr, CanisterPtr)>,
    is_maintenance_active: bool,
    creating_data_canister: bool,
    creating_search_canister: bool,
    batch_limit_bytes: u64,
    canister_available_queue: VecDeque<CanisterId>,
    used_bytes_threshold: u32,
    used_bytes_total: u64,
    search_canisters: Vec<CanisterId>,
    data_bucket_canister_wasm_binary: Vec<u8>,
    search_canister_wasm_binary: Vec<u8>,
    id: CanisterId,
    // Testing functions
    #[cfg(not(target_arch = "wasm32"))]
    fn_ptr_add_to_search_index: Option<FnPtrAddToSearchIndex>,
    #[cfg(not(target_arch = "wasm32"))]
    fn_ptr_batch_add_to_search_index: Option<FnPtrBatchAddToSearchIndex>,
    #[cfg(not(target_arch = "wasm32"))]
    fn_ptr_search_keys_by_query: Option<FnPtrSearchKeysByQuery>,
    #[cfg(not(target_arch = "wasm32"))]
    fn_ptr_remove_from_search_index: Option<FnPtrRemoveFromSearchIndex>,
    #[cfg(not(target_arch = "wasm32"))]
    fn_ptr_list: Option<FnPtrList>,
    #[cfg(not(target_arch = "wasm32"))]
    fn_ptr_used_bytes: Option<FnPtrUsedBytes>,
    #[cfg(not(target_arch = "wasm32"))]
    fn_ptr_holds_key: Option<Box<dyn Fn(CanisterId, &Key) -> bool>>,
    #[cfg(not(target_arch = "wasm32"))]
    fn_ptr_set_range: Option<FnPtrSetRange>,
    #[cfg(not(target_arch = "wasm32"))]
    fn_ptr_get_relocation_batch: Option<FnPtrGetRelocationBatch>,
    #[cfg(not(target_arch = "wasm32"))]
    fn_ptr_put_relocation_batch: Option<FnPtrPutRelocationBatch>,
    #[cfg(not(target_arch = "wasm32"))]
    fn_ptr_delete_entries: Option<FnPtrDeleteEntries>,
}

#[allow(dead_code)]
impl BigmapIdx {
    pub fn new() -> Self {
        let mut result: BigmapIdx = BigmapIdx::default();
        result.reset();
        result
    }

    pub fn reset(&mut self) {
        *self = Self {
            used_bytes_threshold: 3 * 1024 * 1024 * 1024,
            batch_limit_bytes: 1024 * 1024,
            ..Default::default()
        }
    }

    pub async fn get(&self, key: &Key) -> Option<Val> {
        match self.lookup_get(&key).await {
            Some(can_id) => {
                let can_id: ic_cdk::CanisterId = can_id.0.into();
                println!(
                    "BigMap Index: get key {} @CanisterId {}",
                    String::from_utf8_lossy(&key),
                    can_id
                );
                ic_cdk::call(can_id, "get", Some(key)).await.unwrap()
            }
            None => {
                println!(
                    "BigMap Index: no data canister holds the key {}",
                    String::from_utf8_lossy(&key)
                );
                None
            }
        }
    }

    pub async fn put(&self, key: &Key, value: &Val) -> u64 {
        match self.lookup_put(&key) {
            Some(can_id) => {
                let can_id: ic_cdk::CanisterId = can_id.0.into();
                match ic_cdk::call(can_id.clone(), "batch_put", Some(vec![(key, value)]))
                    .await
                    .expect(&format!(
                        "BigMap index: put call to CanisterId {} failed",
                        can_id
                    )) {
                    1u64 => value.len() as u64,
                    _ => 0,
                }
            }
            None => {
                println!(
                    "BigMap Index: no data canister suitable for key {}",
                    String::from_utf8_lossy(&key)
                );
                0
            }
        }
    }

    pub async fn batch_put(&self, batch: &Vec<(Key, Val)>) -> u64 {
        let mut result = 0;
        let mut batches: DetHashMap<CanisterId, Vec<(Key, Val)>> = DetHashMap::default();
        for (key, value) in batch.into_iter() {
            match self.lookup_put(&key) {
                Some(can_id) => {
                    if batches.contains_key(&can_id) {
                        batches
                            .get_mut(&can_id)
                            .unwrap()
                            .push((key.clone(), value.clone()));
                    } else {
                        batches.insert(can_id, vec![(key.clone(), value.clone())]);
                    };
                }
                None => {
                    println!(
                        "BigMap Index: no data canister suitable for key {}",
                        String::from_utf8_lossy(&key)
                    );
                }
            }
        }
        for (can_id, batch) in batches.into_iter() {
            let can_id: ic_cdk::CanisterId = can_id.0.into();
            let batch_result: u64 = ic_cdk::call(can_id.clone(), "batch_put", Some(batch))
                .await
                .expect(&format!(
                    "BigMap index: put call to CanisterId {} failed",
                    can_id
                ));
            result += batch_result;
        }
        result
    }

    pub async fn append(&mut self, key: &Key, value: &Val) -> u64 {
        if let Err(err) = self.ensure_at_least_one_data_canister().await {
            println!(
                "Error appending key {} => {}",
                String::from_utf8_lossy(key),
                err
            );
            return 0;
        }

        match self.lookup_put(&key) {
            Some(can_id) => {
                let can_id: ic_cdk::CanisterId = can_id.0.into();
                println!(
                    "BigMap Index: append key {} @CanisterId {}",
                    String::from_utf8_lossy(&key),
                    can_id
                );
                ic_cdk::call(can_id.clone(), "append_from_index", Some((key, value)))
                    .await
                    .expect(&format!(
                        "BigMap index: append call to CanisterId {} failed",
                        can_id
                    ))
            }
            None => {
                println!(
                    "BigMap Index: no data canister suitable for key {}",
                    String::from_utf8_lossy(&key)
                );
                0
            }
        }
    }

    fn can_ptr_to_canister_id(&self, can_ptr: &CanisterPtr) -> CanisterId {
        self.idx[can_ptr.0 as usize].clone()
    }

    pub async fn add_canisters(&mut self, can_ids: Vec<CanisterId>) {
        // let mut new_can_util_vec = Vec::new();

        for can_id in can_ids {
            for can_id_existing in &self.idx {
                if &can_id == can_id_existing {
                    println!(
                        "BigMap Index: Skipping already existing Data CanisterId {}",
                        can_id
                    );
                }
            }
            for can_id_existing in &self.canister_available_queue {
                if &can_id == can_id_existing {
                    println!(
                        "BigMap Index: Skipping already existing Data CanisterId {}",
                        can_id
                    );
                }
            }

            println!("BigMap Index: Created Data CanisterId {}", can_id);

            // Add all canisters to the available queue
            self.canister_available_queue.push_back(can_id);
        }

        if let Err(err) = self.ensure_at_least_one_data_canister().await {
            self.is_maintenance_active = false;
            println!("Error adding canisters: {}", err);
        }
    }

    pub async fn ensure_at_least_one_data_canister(&mut self) -> Result<(), String> {
        if self.hash_ring.is_empty() {
            if self.creating_data_canister {
                return Err(
                    "Already creating data canister, concurrent calls are not allowed".to_string(),
                );
            }
            self.creating_data_canister = true;
            println!("BigMap Index: No Data Canisters, creating one!");
            match self.create_data_bucket_canister().await {
                Ok(can_id) => {
                    println!("BigMap Index: Activating Data CanisterId {}", can_id);

                    let range = self.hash_ring_add_canister_id(&can_id);
                    self.ucall_dcan_set_range(&can_id, range.0, range.1).await;
                }
                Err(err) => {
                    println!("BigMap Index: Error creating a new Data Canister {}", err);
                }
            }
            self.creating_data_canister = false;
        };
        Ok(())
    }

    pub async fn ensure_at_least_one_search_canister(&mut self) -> Result<(), String> {
        if self.search_canisters.is_empty() {
            if self.creating_search_canister {
                return Err(
                    "Already creating search canister, concurrent calls are not allowed"
                        .to_string(),
                );
            }
            self.creating_search_canister = true;
            println!("BigMap Index: No Search Canisters, creating one!");
            match self.create_search_canister().await {
                Ok(can_id) => {
                    println!("BigMap Index: Activating Search CanisterId {}", can_id);
                    self.search_canisters.push(can_id);
                }
                Err(err) => {
                    println!("BigMap Index: Error creating a new Search Canister {}", err);
                }
            }
            self.creating_search_canister = false;
        };
        Ok(())
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
        // println!("BigMap Index: lookup_get @key {}", String::from_utf8_lossy(key));

        let can_id = self.can_ptr_to_canister_id(can_ptr);
        if self.qcall_dcan_holds_key(&can_id, key).await {
            return Some(can_id);
        }

        if let Some(can_src_dst_ptr) = self.now_rebalancing_src_dst {
            let (rebalance_src_ptr, rebalance_dst_ptr) = can_src_dst_ptr;
            if can_ptr == &rebalance_src_ptr {
                // The destination canister doesn't have the key but it's currently rebalancing.
                // The key may not have been moved yet from the source canister

                let can_id = self.can_ptr_to_canister_id(&rebalance_dst_ptr);
                if self.qcall_dcan_holds_key(&can_id, key).await {
                    println!(
                        "BigMap Index: lookup_get @key {} from a relocation destination {}",
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

        // println!("BigMap Index: lookup_put @key {}", String::from_utf8_lossy(key));
        Some(self.can_ptr_to_canister_id(ring_node))
    }

    // List keys starting with key_prefix
    pub async fn list(&self, key_prefix: &Key) -> Vec<Key> {
        let mut result = BTreeSet::new();

        for can_id in self.idx.iter() {
            let sub_list: Vec<Key> = self.qcall_dcan_list(can_id, key_prefix).await;
            result.extend(sub_list);
            if result.len() > 10000 {
                // Safety brake, don't return too many entries
                break;
            }
        }

        result.iter().cloned().collect()
    }

    pub async fn maintenance(&mut self) -> String {
        #[derive(serde::Serialize)]
        struct Status {
            status: &'static str,
            message: &'static str,
        };

        if self.is_maintenance_active {
            return serde_json_wasm::to_string(&Status {
                status: "Good",
                message: "Already rebalancing",
            })
            .unwrap();
        }
        self.is_maintenance_active = true;

        if let Err(_) = self.ensure_at_least_one_data_canister().await {
            self.is_maintenance_active = false;
            return serde_json_wasm::to_string(&Status {
                status: "Unknown",
                message: "Error trying to ensure at least one data canister",
            })
            .unwrap();
        }

        println!("BigMap Index: starting maintenance");

        self.used_bytes_total = 0;

        for i in 0..self.idx.len() {
            let can_id = self.idx[i].clone();
            let can_ptr = CanisterPtr { 0: i as u32 };
            let used_bytes = self.qcall_canister_used_bytes(&can_id).await as u64;
            self.used_bytes_total += used_bytes;

            self.print_canister_utilization(&can_id, used_bytes);

            if used_bytes as u32 > self.used_bytes_threshold {
                println!(
                    "BigMap Index: CanisterId {} used bytes {} is over threshold {}",
                    can_id, used_bytes, self.used_bytes_threshold
                );

                // This canister should be rebalanced. We'll do these steps:
                // - Create destination canister, to which half of the data from the source canister will go
                // - Move batches of objects from source canister to the destination canister
                // We're just starting to rebalance, create the destination canister
                let src_canister_ptr = can_ptr.clone();
                let src_canister = self.can_ptr_to_canister_id(&src_canister_ptr);
                let dst_canister = self
                    .create_data_bucket_canister()
                    .await
                    .expect("create_data_bucket_canister failed");

                let (dst_canister_ptr, range_dst, range_src) =
                    self.hash_ring_add_before_this(&src_canister_ptr, &dst_canister.clone());

                // The new canister has been created and added to the hash ring
                // Remember the canisters we're currently rebalancing
                self.now_rebalancing_src_dst = Some((src_canister_ptr, dst_canister_ptr));

                // Update the range covered by Source and Destination canister
                self.ucall_dcan_set_range(&dst_canister, range_dst.0, range_dst.1)
                    .await;
                self.ucall_dcan_set_range(&src_canister, range_src.0, range_src.1)
                    .await;

                // Start moving data
                let (rebalance_src_ptr, rebalance_dst_ptr) = self.now_rebalancing_src_dst.unwrap();
                let src_canister = self.can_ptr_to_canister_id(&rebalance_src_ptr);
                let dst_canister = self.can_ptr_to_canister_id(&rebalance_dst_ptr);

                loop {
                    let batch = self
                        .ucall_dcan_get_relocation_batch(&src_canister, self.batch_limit_bytes)
                        .await;

                    println!(
                        "BigMap Index: Got relocation batch of {} entries",
                        batch.len()
                    );

                    if batch.is_empty() {
                        // Finished rebalancing this canister
                        self.now_rebalancing_src_dst = None;
                        break;
                    } else {
                        let put_count = self
                            .ucall_dcan_put_relocation_batch(&dst_canister, &batch)
                            .await;
                        if batch.len() as u64 != put_count {
                            println!(
                                "BigMap Index: Not all elements were moved from {} to {}",
                                src_canister, dst_canister
                            )
                        } else {
                            println!(
                                "BigMap Index: Moved {} elements from {} to {}",
                                batch.len(),
                                src_canister,
                                dst_canister
                            )
                        }
                        let batch_sha2 = batch.iter().map(|e| e.0.clone()).collect();

                        self.ucall_dcan_delete_entries(&src_canister, &batch_sha2)
                            .await;
                    }
                }
            }
        }

        // FIXME: Check the utilization of the Search canisters, split if necessary
        // FIXME: Remove and/or update the indexes in the Search canisters

        for can_id in self.search_canisters.iter() {
            let used_bytes = self.qcall_canister_used_bytes(can_id).await as u32;
            self.used_bytes_total += used_bytes as u64;
        }

        println!("Total capacity used {}", ByteSize(self.used_bytes_total));

        self.is_maintenance_active = false;

        serde_json_wasm::to_string(&Status {
            status: "Good",
            message: "Finished maintenance",
        })
        .unwrap()
    }

    pub async fn status(&self) -> String {
        #[derive(serde::Serialize, Default)]
        struct DataBucketStatus {
            canister_id: String,
            used_bytes: u32,
        };

        #[derive(serde::Serialize, Default)]
        struct SearchCanisterStatus {
            canister_id: String,
            used_bytes: u32,
        };

        #[derive(serde::Serialize, Default)]
        struct Status {
            data_buckets: Vec<DataBucketStatus>,
            search_canisters: Vec<SearchCanisterStatus>,
            used_bytes_total: u64,
        };

        let mut status = Status::default();

        for can_id in self.idx.iter() {
            let used_bytes = self.qcall_canister_used_bytes(can_id).await as u32;
            status.data_buckets.push(DataBucketStatus {
                canister_id: can_id.to_string(),
                used_bytes,
            });
            status.used_bytes_total += used_bytes as u64;
        }

        for can_id in self.search_canisters.iter() {
            let used_bytes = self.qcall_canister_used_bytes(can_id).await as u32;
            status.search_canisters.push(SearchCanisterStatus {
                canister_id: can_id.to_string(),
                used_bytes,
            });
            status.used_bytes_total += used_bytes as u64;
        }

        serde_json_wasm::to_string(&status).unwrap()
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

    pub fn set_used_bytes_threshold(&mut self, used_bytes_threshold: u32) {
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
            None => match subnet_create_new_canister().await {
                Ok(new_can_id) => {
                    println!("BigMap Index: Created new CanisterId {}", new_can_id);
                    match subnet_install_canister_code(
                        new_can_id.clone(),
                        self.data_bucket_canister_wasm_binary.clone(),
                    )
                    .await
                    {
                        Ok(_) => {
                            println!("BigMap Index: Code install successful to {}", new_can_id)
                        }
                        Err(err) => println!(
                            "CanisterId {}: code install failed with error {}",
                            new_can_id, err
                        ),
                    };
                    Ok(new_can_id)
                }
                Err(err) => Err(err),
            },
        }
    }

    async fn create_search_canister(&mut self) -> Result<CanisterId, String> {
        match subnet_create_new_canister().await {
            Ok(new_can_id) => {
                println!("BigMap Index: Created new CanisterId {}", new_can_id);
                match subnet_install_canister_code(
                    new_can_id.clone(),
                    self.search_canister_wasm_binary.clone(),
                )
                .await
                {
                    Ok(_) => println!("BigMap Index: Code install successful to {}", new_can_id),
                    Err(err) => println!(
                        "CanisterId {}: code install failed with error {}",
                        new_can_id, err
                    ),
                };
                Ok(new_can_id)
            }
            Err(err) => Err(err),
        }
    }

    pub async fn set_data_bucket_canister_wasm_binary(&mut self, wasm_binary: Vec<u8>) {
        self.data_bucket_canister_wasm_binary = wasm_binary;
        if let Err(err) = self.ensure_at_least_one_data_canister().await {
            println!("Error adding canisters: {}", err);
        }
    }

    pub async fn set_search_canister_wasm_binary(&mut self, wasm_binary: Vec<u8>) {
        self.search_canister_wasm_binary = wasm_binary;
        if let Err(err) = self.ensure_at_least_one_search_canister().await {
            println!("Error adding canisters: {}", err);
        }
    }

    fn print_canister_utilization(&self, can_id: &CanisterId, used_bytes: u64) {
        println!(
            "CanisterId {} used {}",
            can_id.clone(),
            ByteSize(used_bytes)
        );
    }

    // Returns a randomly generated and unused key
    pub async fn get_random_key(&self) -> String {
        let time_bytes = ic_cdk::time().to_be_bytes();
        let mut rand_key = calc_sha256(&time_bytes.to_vec());
        for i in 0..100u32 {
            // Only try this a limited number of times
            let rand_key_hash = calc_sha256(&rand_key);
            let (_, can_ptr) = match self.hash_ring.get_idx_node_for_key(&rand_key_hash) {
                Some(v) => v,
                None => return hex::encode(rand_key),
            };

            let can_id = self.can_ptr_to_canister_id(can_ptr);

            let key_is_used = self
                .qcall_dcan_holds_key(&can_id, &Vec::from(rand_key.as_slice()))
                .await;

            if !key_is_used {
                let result = hex::encode(rand_key);
                println!(
                    "get_random_key: after {} attempts found {} which maps to {}",
                    i, result, can_id
                );
                return result;
            }

            rand_key = rand_key_hash;
        }
        println!("get_random_key: failed to find an unused key in the range");
        "".to_string()
    }

    //
    // Search functions
    //

    pub async fn put_and_fts_index(&mut self, key: &Key, document: &String) -> u64 {
        if let Err(err) = self.ensure_at_least_one_search_canister().await {
            println!(
                "Error putting key {} => {}",
                String::from_utf8_lossy(key),
                err
            );
            return 0;
        }

        let value_vec = Vec::from(document.as_bytes());

        let result = self.put(&key, &value_vec).await;

        // FIXME: Ensure the search canister has enough space and allocate a new one if necessary
        let search_can_id = &self.search_canisters[0].clone();
        self.ucall_s_can_add_to_search_index(&search_can_id, key, document)
            .await;

        result
    }

    pub async fn batch_put_and_fts_index(&mut self, batch: &Vec<(Key, String)>) -> u64 {
        if let Err(err) = self.ensure_at_least_one_search_canister().await {
            println!("Error putting a batch of length {} => {}", batch.len(), err);
            return 0;
        }

        let batch_as_bytes: Vec<_> = batch
            .iter()
            .map(|(k, v)| (k.clone(), Vec::from(v.as_bytes())))
            .collect();

        self.batch_put(&batch_as_bytes).await;

        // FIXME: Ensure the search canister has enough space and allocate a new one if necessary
        let search_can_id = &self.search_canisters[0].clone();
        self.ucall_s_can_batch_add_to_search_index(&search_can_id, batch)
            .await;

        batch.len() as u64
    }

    pub async fn remove_from_fts_index(&mut self, key: &Key) {
        if let Err(err) = self.ensure_at_least_one_search_canister().await {
            println!(
                "Error removing key {} => {}",
                String::from_utf8_lossy(key),
                err
            );
            return;
        }

        for can_id in self.search_canisters.iter() {
            self.ucall_s_can_remove_from_search_index(can_id, key).await
        }
    }

    pub async fn search(&self, search_query: &String) -> (u64, Vec<(Key, Val)>) {
        if self.search_canisters.is_empty() {
            return (0, Vec::new());
        }

        let mut results = Vec::new();
        let mut results_len = 0;

        for can_id in self.search_canisters.iter() {
            let results_per_canister = self
                .qcall_s_can_search_keys_by_query(can_id, search_query)
                .await;
            results_len += results_per_canister.len() as u64;
            for key in results_per_canister {
                match self.get(&key).await {
                    Some(value) => {
                        println!(
                            "search {} => key {} value {}",
                            search_query,
                            String::from_utf8_lossy(&key),
                            String::from_utf8_lossy(&value)
                        );
                        results.push((key, value));
                        if results.len() >= 20 {
                            return (results_len, results);
                        }
                    }
                    None => continue,
                }
            }
        }

        (results_len, results)
    }
}

#[cfg(target_arch = "wasm32")]
impl BigmapIdx {
    async fn ucall_s_can_batch_add_to_search_index(
        &self,
        can_id: &CanisterId,
        doc_vec: &Vec<(Key, String)>,
    ) -> u64 {
        ic_cdk::call(
            can_id.clone().0.into(),
            "batch_add_to_search_index",
            Some(doc_vec),
        )
        .await
        .expect("batch_add_to_search_index call failed")
    }

    async fn ucall_s_can_add_to_search_index(
        &self,
        can_id: &CanisterId,
        key: &Key,
        document: &String,
    ) {
        ic_cdk::call_no_return(
            can_id.clone().0.into(),
            "add_to_search_index",
            Some((key, document)),
        )
        .await
        .expect("add_to_search_index call failed")
    }

    async fn ucall_s_can_remove_from_search_index(&self, can_id: &CanisterId, key: &Key) {
        ic_cdk::call_no_return(
            can_id.clone().0.into(),
            "remove_from_search_index",
            Some(key),
        )
        .await
        .expect("remove_from_search_index call failed")
    }

    async fn qcall_s_can_search_keys_by_query(
        &self,
        can_id: &CanisterId,
        search_query: &String,
    ) -> Vec<Key> {
        ic_cdk::call(
            can_id.clone().0.into(),
            "search_keys_by_query",
            Some(search_query),
        )
        .await
        .expect("search_keys_by_query call failed")
    }

    async fn qcall_dcan_list(&self, can_id: &CanisterId, key_prefix: &Vec<u8>) -> Vec<Key> {
        ic_cdk::call(can_id.clone().0.into(), "list", Some(key_prefix))
            .await
            .expect("list call failed")
    }

    async fn qcall_canister_used_bytes(&self, can_id: &CanisterId) -> usize {
        ic_cdk::call(can_id.clone().0.into(), "used_bytes", Some(()))
            .await
            .expect("used_bytes call failed")
    }

    async fn qcall_dcan_holds_key(&self, can_id: &CanisterId, key: &Key) -> bool {
        ic_cdk::call(can_id.clone().0.into(), "holds_key", Some(key))
            .await
            .expect("holds_key call failed")
    }

    async fn ucall_dcan_set_range(
        &self,
        can_id: &CanisterId,
        range_start: Sha256Digest,
        range_end: Sha256Digest,
    ) {
        ic_cdk::call_no_return(
            can_id.clone().0.into(),
            "set_range",
            Some((range_start.to_vec(), range_end.to_vec())),
        )
        .await
        .expect("set_range call failed")
    }

    async fn ucall_dcan_get_relocation_batch(
        &self,
        can_id: &CanisterId,
        batch_size_bytes: u64,
    ) -> Vec<(Sha2Vec, Key, Val)> {
        ic_cdk::call(
            can_id.clone().0.into(),
            "get_relocation_batch",
            Some(batch_size_bytes),
        )
        .await
        .expect("get_relocation_batch call failed")
    }

    async fn ucall_dcan_put_relocation_batch(
        &self,
        can_id: &CanisterId,
        batch: &Vec<(Sha2Vec, Key, Val)>,
    ) -> u64 {
        ic_cdk::call(can_id.clone().0.into(), "put_relocation_batch", Some(batch))
            .await
            .expect("put_relocation_batch call failed")
    }

    async fn ucall_dcan_delete_entries(&self, can_id: &CanisterId, keys_sha2: &Vec<Vec<u8>>) {
        ic_cdk::call_no_return(can_id.clone().0.into(), "delete_entries", Some(keys_sha2))
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

    pub fn set_fn_ptr_add_to_search_index(&mut self, fn_ptr: FnPtrAddToSearchIndex) {
        self.fn_ptr_add_to_search_index = Some(fn_ptr);
    }

    pub fn set_fn_ptr_remove_from_search_index(&mut self, fn_ptr: FnPtrRemoveFromSearchIndex) {
        self.fn_ptr_remove_from_search_index = Some(fn_ptr);
    }

    pub fn set_fn_ptr_search_keys_by_query(&mut self, fn_ptr: FnPtrSearchKeysByQuery) {
        self.fn_ptr_search_keys_by_query = Some(fn_ptr);
    }

    pub fn set_fn_ptr_used_bytes(&mut self, fn_ptr: FnPtrUsedBytes) {
        self.fn_ptr_used_bytes = Some(fn_ptr);
    }

    pub fn set_fn_ptr_set_range(&mut self, fn_ptr: FnPtrSetRange) {
        self.fn_ptr_set_range = Some(fn_ptr);
    }

    pub fn set_fn_ptr_list(&mut self, fn_ptr: FnPtrList) {
        self.fn_ptr_list = Some(fn_ptr);
    }

    pub fn set_fn_ptr_holds_key(&mut self, fn_ptr: FnPtrHoldsKey) {
        self.fn_ptr_holds_key = Some(fn_ptr);
    }

    pub fn set_fn_ptr_get_relocation_batch(&mut self, fn_ptr: FnPtrGetRelocationBatch) {
        self.fn_ptr_get_relocation_batch = Some(fn_ptr);
    }

    pub fn set_fn_ptr_put_relocation_batch(&mut self, fn_ptr: FnPtrPutRelocationBatch) {
        self.fn_ptr_put_relocation_batch = Some(fn_ptr);
    }

    pub fn set_fn_ptr_delete_entries(&mut self, fn_ptr: FnPtrDeleteEntries) {
        self.fn_ptr_delete_entries = Some(fn_ptr);
    }

    async fn ucall_s_can_batch_add_to_search_index(
        &self,
        can_id: &CanisterId,
        doc_vec: &Vec<(Key, String)>,
    ) -> u64 {
        let fn_ptr = self
            .fn_ptr_batch_add_to_search_index
            .as_ref()
            .expect("fn_ptr_batch_add_to_search_index is not set");
        fn_ptr(can_id.clone(), doc_vec)
    }

    async fn ucall_s_can_add_to_search_index(
        &self,
        can_id: &CanisterId,
        key: &Vec<u8>,
        doc: &String,
    ) {
        let fn_ptr = self
            .fn_ptr_add_to_search_index
            .as_ref()
            .expect("fn_ptr_add_to_search_index is not set");
        fn_ptr(can_id.clone(), key, doc)
    }

    async fn ucall_s_can_remove_from_search_index(&self, can_id: &CanisterId, key: &Vec<u8>) {
        let fn_ptr = self
            .fn_ptr_remove_from_search_index
            .as_ref()
            .expect("fn_ptr_remove_from_search_index is not set");
        fn_ptr(can_id.clone(), key)
    }

    async fn qcall_s_can_search_keys_by_query(
        &self,
        can_id: &CanisterId,
        search_query: &String,
    ) -> Vec<Key> {
        let fn_ptr = self
            .fn_ptr_search_keys_by_query
            .as_ref()
            .expect("fn_ptr_search_keys_by_query is not set");
        fn_ptr(can_id.clone(), search_query)
    }

    async fn qcall_canister_used_bytes(&self, can_id: &CanisterId) -> usize {
        let fn_ptr = self
            .fn_ptr_used_bytes
            .as_ref()
            .expect("fn_ptr_used_bytes is not set");
        fn_ptr(can_id.clone())
    }

    async fn qcall_dcan_list(&self, can_id: &CanisterId, key_prefix: &Vec<u8>) -> Vec<Key> {
        let fn_ptr = self.fn_ptr_list.as_ref().expect("fn_ptr_list is not set");
        fn_ptr(can_id.clone(), key_prefix)
    }

    async fn qcall_dcan_holds_key(&self, can_id: &CanisterId, key: &Key) -> bool {
        let fn_ptr = self
            .fn_ptr_holds_key
            .as_ref()
            .expect("fn_ptr_used_bytes is not set");
        fn_ptr(can_id.clone(), key)
    }

    async fn ucall_dcan_set_range(
        &self,
        can_id: &CanisterId,
        range_start: Sha256Digest,
        range_end: Sha256Digest,
    ) {
        let fn_ptr = self
            .fn_ptr_set_range
            .as_ref()
            .expect("fn_ptr_set_range is not set");
        fn_ptr(can_id.clone(), range_start, range_end)
    }

    async fn ucall_dcan_get_relocation_batch(
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

    async fn ucall_dcan_put_relocation_batch(
        &self,
        can_id: &CanisterId,
        batch: &Vec<(Sha2Vec, Key, Val)>,
    ) -> u64 {
        let fn_ptr = self
            .fn_ptr_put_relocation_batch
            .as_ref()
            .expect("fn_ptr_put_relocation_batch is not set");
        fn_ptr(can_id.clone(), batch)
    }

    async fn ucall_dcan_delete_entries(&self, can_id: &CanisterId, keys_sha2: &Vec<Vec<u8>>) {
        let fn_ptr = self
            .fn_ptr_delete_entries
            .as_ref()
            .expect("fn_ptr_delete_entries is not set");
        fn_ptr(can_id.clone(), keys_sha2)
    }
}

#[cfg(test)]
mod tests;
