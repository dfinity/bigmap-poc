use crate::{hashring, println, CanisterId, Key};
use min_max_heap::MinMaxHeap;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::hash::BuildHasherDefault;
use wyhash::WyHash;

// CanisterPtr allows us to have u64 instead of a full CanisterId
// in various parts of the BigMap Index
#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct CanisterPtr(u32);
pub type DetHashSet<K> = HashSet<K, BuildHasherDefault<WyHash>>;

bitflags::bitflags! {
    #[derive(Default)]
    struct Flags: u8 {
        const USAGE_ACCURATE = 0b0000_0001;
        const REBALANCING = 0b0000_0010;
        // const C = 0b00000100;
        // const ABC = Self::A.bits | Self::B.bits | Self::C.bits;
    }
}

// #[derive(Default)]
// struct CanStats {
//     state: CanState,
//     used_bytes: u32,
// }

#[allow(dead_code)]
#[derive(Default)]
pub struct BigmapIdx {
    idx: Vec<CanisterId>, // indirection for CanisterId, to avoid many copies of CanisterIds
    flags: Vec<Flags>,
    can_ring: hashring::HashRing<CanisterPtr>,
    utilization_heap: MinMaxHeap<CanisterUsedBytes>, // Canisters that can be rebalanced
    can_rebalancing: DetHashSet<CanisterPtr>,
    can_query_util: DetHashSet<CanisterPtr>,
    can_needed: u32, // how many canisters need to be created?
    xcq_used_bytes: Option<Box<dyn Fn(CanisterId) -> usize + Send>>, // cross-canister query fn
    xcq_holds_key: Option<Box<dyn Fn(CanisterId, &Key) -> bool + Send>>, // cross-canister query fn
    id: CanisterId,
}

#[allow(dead_code)]
impl BigmapIdx {
    pub fn new() -> Self {
        Self {
            can_needed: 3,
            ..Default::default()
        }
    }

    pub fn canisters_needed(&self) -> u32 {
        self.can_needed
    }

    fn can_ptr_to_canister_id(&self, can_ptr: &CanisterPtr) -> CanisterId {
        self.idx[can_ptr.0 as usize].clone()
    }

    pub fn add_canisters(&mut self, can_ids: Vec<CanisterId>) {
        // let mut new_can_util_vec = Vec::new();

        for can_id in can_ids {
            // println!("BigMap Index: add data canister_id={}", can_id);

            let ptr_new = CanisterPtr {
                0: self.idx.len() as u32,
            };
            let mut ptr_next = None;
            self.idx.push(can_id);
            self.flags.push(Flags::default());

            match self.utilization_heap.pop_max() {
                Some(v) => {
                    // This canister can be rebalanced
                    let (e_idx, e_node_next) = self.can_ring.get_idx_node(&v.ring_key).unwrap();
                    ptr_next = Some(e_node_next.clone());
                    let (e_key, _) = self.can_ring.get_key_node_at_idx(e_idx).unwrap();
                    let (e_key_prev, _) = self.can_ring.get_prev_key_node_at_idx(e_idx).unwrap();
                    let e_key_new = (e_key - e_key_prev) / 2;
                    self.can_ring.add_with_key(e_key_new, ptr_new.clone());
                }
                // No known canister to be rebalanced, add at an arbitrary position
                None => self.can_ring.add(ptr_new.clone()),
            };

            // This canister may not hold all the data it should hold
            if let Some(ptr_next) = ptr_next {
                // Data from the next canister may need to be rebalanced into this one
                let xcq_used_bytes = self
                    .xcq_used_bytes
                    .as_ref()
                    .expect("xcq_used_bytes is not set");
                if xcq_used_bytes(self.idx[ptr_next.0 as usize].clone()) > 0 {
                    self.flags[ptr_new.0 as usize] |= Flags::REBALANCING;
                    self.can_rebalancing.insert(ptr_new);
                    self.flags[ptr_next.0 as usize] |= Flags::REBALANCING;
                    self.can_rebalancing.insert(ptr_next);
                }
            }

            // self.can_ring.add_with_key(max_utilized_can.ring_key, can_id);
            if self.can_needed > 0 {
                self.can_needed -= 1;
            }
        }

        self.utilization_heap.clear(); // we need fresh data
    }

    // Returns the CanisterIds which holds the key
    // If multiple canisters can hold the data due to rebalancing, we will
    // query all candidates and return the correct CanisterId
    pub fn lookup_get(&self, key: &Key) -> CanisterId {
        let (ring_idx, ring_node) = self.can_ring.get_idx_node(key).unwrap();

        let xcq_holds_key = self
            .xcq_holds_key
            .as_ref()
            .expect("xcq_holds_key is not set");

        let can_id = self.can_ptr_to_canister_id(ring_node);
        if xcq_holds_key(can_id.clone(), key) {
            return can_id;
        }
        if self.can_rebalancing.contains(&ring_node) {
            if let Some((_, ring_node_next)) = self.can_ring.get_next_key_node_at_idx(ring_idx) {
                let can_id = self.can_ptr_to_canister_id(ring_node_next);
                if xcq_holds_key(can_id.clone(), key) {
                    return can_id;
                }
            }
        }
        Default::default()
    }

    // Returns the CanisterIds which holds the key
    // If multiple canisters can hold the data due to rebalancing, we will
    // query all candidates and return the correct CanisterId
    pub fn lookup_put(&self, key: &Key) -> CanisterId {
        let (_, ring_node) = self.can_ring.get_idx_node(key).unwrap();

        // println!("BM index lookup_put @key {}", String::from_utf8_lossy(key));
        self.can_ptr_to_canister_id(ring_node)
    }

    pub fn rebalance(&mut self) -> Result<u8, String> {
        println!(
            "BigMap Index: rebalance pending can_ids {:?}",
            self.can_rebalancing
        );

        let xcq_used_bytes = self
            .xcq_used_bytes
            .as_ref()
            .expect("xcq_used_bytes is not set");

        for (i, can_id) in self.idx.iter().enumerate() {
            // let can_ptr = CanisterPtr { 0: i as u32 };
            let used_bytes = xcq_used_bytes(can_id.clone());
            if self.flags[i].contains(Flags::REBALANCING) {
                println!("used: can_id {} -> {} (rebalancing)", can_id, used_bytes);
            } else {
                println!("used: can_id {} -> {}", can_id, used_bytes);
            }
        }

        self.can_rebalancing.clear();

        Ok(0)
    }

    pub fn set_canister_id(&mut self, can_id: CanisterId) {
        self.id = can_id
    }

    pub fn canister_id(&self) -> CanisterId {
        self.id.clone()
    }
}

// Cross-canister calls - function pointers
// We can remove these once the Rust SDK is able to run Rust canisters natively
impl BigmapIdx {
    pub fn set_fn_xcq_used_bytes(&mut self, fn_ptr: Box<dyn Fn(CanisterId) -> usize + Send>) {
        self.xcq_used_bytes = Some(fn_ptr);
    }

    pub fn set_fn_xcq_holds_key(&mut self, fn_ptr: Box<dyn Fn(CanisterId, &Key) -> bool + Send>) {
        self.xcq_holds_key = Some(fn_ptr);
    }
}

#[derive(Clone, Eq, PartialEq)]
struct CanisterUsedBytes {
    used_bytes: u32,
    ring_key: CanisterPtr,
}

impl Ord for CanisterUsedBytes {
    fn cmp(&self, other: &CanisterUsedBytes) -> Ordering {
        // In case of a tie compare ring_key - this step is necessary
        // to make implementations of `PartialEq` and `Ord` consistent.
        self.used_bytes
            .cmp(&other.used_bytes)
            .then_with(|| self.ring_key.cmp(&other.ring_key))
    }
}

// `PartialOrd` needs to be implemented as well.
impl PartialOrd for CanisterUsedBytes {
    fn partial_cmp(&self, other: &CanisterUsedBytes) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod tests;
