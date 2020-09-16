///////////////////////////////////////////////////////////////
// Consistent Hash implementation, based on the HashRing crate
// https://github.com/jeromefroe/hashring-rs/
// but using SHA-256 for keys
///////////////////////////////////////////////////////////////
use crate::Sha256Digest;
use num_bigint::BigUint;
use std::cmp::Ordering;

// Node is an internal struct used to encapsulate the nodes that will be added
// and removed from `HashRing`
#[derive(Debug, Clone)]
pub(crate) struct Node<T: Clone> {
    pub(crate) key: Sha256Digest,
    pub(crate) node: T,
}

impl<T: Clone> Node<T> {
    fn new(key: Sha256Digest, node: T) -> Node<T> {
        Node { key, node }
    }
}

// Implement `PartialEq`, `Eq`, `PartialOrd` and `Ord` so we can sort `Node`s
impl<T: Clone> PartialEq for Node<T> {
    fn eq(&self, other: &Node<T>) -> bool {
        self.key == other.key
    }
}

impl<T: Clone> Eq for Node<T> {}

impl<T: Clone> PartialOrd for Node<T> {
    fn partial_cmp(&self, other: &Node<T>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Clone> Ord for Node<T> {
    fn cmp(&self, other: &Node<T>) -> Ordering {
        self.key.cmp(&other.key)
    }
}

#[derive(Debug, Default, Clone)]
pub struct HashRing<T: Clone> {
    pub(crate) ring: Vec<Node<T>>,
}

/// Hash Ring
///
/// A hash ring that provides consistent hashing for nodes that are added to it.
#[allow(dead_code)]
impl<T: Clone + Default> HashRing<T> {
    /// Create a new `HashRing`.
    pub fn new() -> HashRing<T> {
        Default::default()
    }
}

#[allow(dead_code)]
impl<T: Clone> HashRing<T> {
    /// Get the number of nodes in the hash ring.
    pub fn len(&self) -> usize {
        self.ring.len()
    }

    /// Returns true if the ring has no elements.
    pub fn is_empty(&self) -> bool {
        self.ring.len() == 0
    }
}

#[inline]
fn biguint_from_slice256(s: &[u8]) -> BigUint {
    BigUint::from_radix_be(s, 256).expect("Converting from Sha256 to BigUint failed")
}

pub fn sha256_digest_to_biguint(val: Sha256Digest) -> BigUint {
    biguint_from_slice256(val.as_slice())
}

pub fn biguint_to_sha256_digest(bigint: &BigUint) -> Sha256Digest {
    let mut key = bigint.to_radix_be(256);
    key.resize(32, 0);
    *Sha256Digest::from_slice(&key)
}

use lazy_static::lazy_static;

lazy_static! {
    pub static ref SHA256_DIGEST_MIN: Sha256Digest = biguint_to_sha256_digest(&BigUint::from(0u64));
    pub static ref SHA256_DIGEST_MAX: Sha256Digest =
        biguint_to_sha256_digest(&BigUint::parse_bytes(b"f".repeat(64).as_slice(), 16).unwrap());
}

pub(crate) fn sha256_range_half(
    sha256_lower: &Sha256Digest,
    sha256_upper: &Sha256Digest,
) -> Sha256Digest {
    let i_prev = biguint_from_slice256(sha256_lower.as_slice());
    let i = biguint_from_slice256(sha256_upper.as_slice());
    let bigint_diff = i_prev.clone() + (i - i_prev) / 2u32;
    biguint_to_sha256_digest(&bigint_diff)
}

#[allow(dead_code)]
impl<T: Clone + PartialEq + std::fmt::Debug> HashRing<T> {
    /// Add `node` to the hash ring.
    /// Returns the index at which the node was added
    pub fn add(&mut self, node: T) -> usize {
        let key = if self.ring.is_empty() {
            BigUint::parse_bytes(b"f".repeat(64).as_slice(), 16).unwrap()
        } else {
            let mut max_gap_idx = 0;
            let mut max_gap_val = biguint_from_slice256(self.ring[max_gap_idx].key.as_slice());
            for (i, node) in self.ring[1..].iter().enumerate() {
                let i = i + 1; // because we skipped the first element of self.ring
                let gap = biguint_from_slice256(node.key.as_slice())
                    - biguint_from_slice256(self.ring[i - 1].key.as_slice());
                if gap > max_gap_val {
                    max_gap_val = gap;
                    max_gap_idx = i;
                }
            }
            if max_gap_idx == 0 {
                biguint_from_slice256(self.ring[max_gap_idx].key.as_slice()) / 2u32
            } else {
                let i_prev = biguint_from_slice256(self.ring[max_gap_idx - 1].key.as_slice());
                let i = biguint_from_slice256(self.ring[max_gap_idx].key.as_slice());
                i_prev.clone() + (i - i_prev) / 2u32
            }
        };
        let key_hash = biguint_to_sha256_digest(&key);
        // let key = get_key(&self.hash_builder, &node);
        self.ring.push(Node::new(key_hash, node.clone()));
        self.ring.sort();
        self.get_idx_key_node_for_node(&node).unwrap().0
    }

    /// Add `node` to the hash ring, with the provided key
    /// Returns the index at which the node was added
    pub fn add_with_key(&mut self, key: &Sha256Digest, node: T) -> usize {
        self.ring.push(Node::new(*key, node.clone()));
        self.ring.sort();
        self.get_idx_key_node_for_node(&node).unwrap().0
    }

    /// Remove `node` from the hash ring. Requires searching the entire ring.
    /// Returns an `Option` that will contain the `node` if it was in the hash
    /// ring or `None` if it was not present.
    pub fn remove_node(&mut self, node: &T) -> Option<T> {
        match self.ring.iter().position(|n| n.node == *node) {
            Some(index) => Some(self.ring.remove(index).node),
            None => None,
        }
    }

    /// Get the Option<(idx,node)> responsible for `key`.
    /// Returns `None` if the ring is empty
    pub fn get_idx_node_for_key(&self, key: &Sha256Digest) -> Option<(usize, &T)> {
        if self.ring.is_empty() {
            return None;
        }

        let n = match self.ring.binary_search_by(|node| node.key.cmp(key)) {
            Err(n) => n,
            Ok(n) => n,
        };

        if n == self.ring.len() {
            return Some((0, &self.ring[0].node));
        }

        Some((n, &self.ring[n].node))
    }

    /// Get the Option<(idx,node)> for the provided `node`.
    /// Returns `None` if the ring is empty or the node couldn't be found
    /// Note that the returned node may be different from the provided, in case there is no exact match
    pub fn get_idx_key_node_for_node(&self, node: &T) -> Option<(usize, Sha256Digest, &T)> {
        if self.ring.is_empty() {
            return None;
        }

        let n = self.ring.iter().position(|e| e.node == *node).unwrap_or(0);

        if n == self.ring.len() {
            // No match found - return the last entry
            return Some((0, self.ring[0].key, &self.ring[0].node));
        }

        Some((n, self.ring[n].key, &self.ring[n].node))
    }

    pub fn get_key_range_for_idx(&self, idx: usize) -> (Sha256Digest, Sha256Digest) {
        if self.ring.is_empty() || idx >= self.ring.len() {
            return (*SHA256_DIGEST_MIN, *SHA256_DIGEST_MIN);
        }

        if idx == 0 {
            return (*SHA256_DIGEST_MIN, self.ring[0].key);
        }
        (self.ring[idx - 1].key, self.ring[idx].key)
    }

    /// Get the Option<(key,node)> responsible for `key`.
    /// Returns `None` if the ring is empty
    pub fn get_key_node(&self, key: &Sha256Digest) -> Option<(Sha256Digest, &T)> {
        match self.get_idx_node_for_key(key) {
            Some((idx, node)) => Some((self.ring[idx].key, node)),
            None => None,
        }
    }

    /// Get the node responsible for `key`. Returns an `Option` that will
    /// contain the `node` if the hash ring is not empty or `None` if it was
    /// empty.
    pub fn get(&self, key: &Sha256Digest) -> Option<&T> {
        match self.get_idx_node_for_key(key) {
            Some((_, node)) => Some(node),
            None => None,
        }
    }

    /// Get the Option<(key,node)> at position `idx`.
    /// Returns `None` if idx is out of bounds
    pub fn get_key_node_at_idx(&self, idx: usize) -> Option<(Sha256Digest, &T)> {
        if idx >= self.ring.len() {
            return None;
        }
        Some((self.ring[idx].key, &self.ring[idx].node))
    }

    /// Get the Option<node> at position `idx + 1`.
    /// Returns `None` if `idx + 1` is out of bounds
    pub fn get_next_key_node_at_idx(&self, idx: usize) -> Option<(Sha256Digest, &T)> {
        match self.get_key_node_at_idx(idx + 1) {
            Some(e) => Some(e),
            None => None,
        }
    }

    /// Get the Option<node> at position `idx - 1`.
    /// Returns `None` if `idx - 1` is out of bounds
    pub fn get_prev_key_node_at_idx(&self, idx: usize) -> Option<(Sha256Digest, &T)> {
        if idx < 1 {
            return None;
        }

        match self.get_key_node_at_idx(idx - 1) {
            Some(e) => Some(e),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::biguint_to_sha256_digest;
    use super::BigUint;
    use super::HashRing;

    #[test]
    fn hashring_sha256_add_key() {
        // Insert key&value pairs and then get the value, and verify the correctness
        let max_hash_uint = BigUint::parse_bytes(b"f".repeat(64).as_slice(), 16).unwrap();
        let max_hash = biguint_to_sha256_digest(&max_hash_uint);
        let max_hash_1_2 = biguint_to_sha256_digest(&(&max_hash_uint / 2u32));
        let max_hash_1_4 = biguint_to_sha256_digest(&(&max_hash_uint / 4u32));
        let max_hash_3_4 = biguint_to_sha256_digest(&(&max_hash_uint * 3u32 / 4u32));

        let mut r = HashRing::new();
        r.add(vec![0u8, 0, 0, 1]);
        let v = r.get_key_node_at_idx(0);
        assert_eq!(v.unwrap().0, max_hash);
        assert_eq!(r.ring.len(), 1);

        r.add(vec![0u8, 0, 0, 2]);
        assert_eq!(r.get_key_node_at_idx(0).unwrap().0, max_hash_1_2);
        assert_eq!(r.get_key_node_at_idx(1).unwrap().0, max_hash);
        assert_eq!(r.ring.len(), 2);

        r.add(vec![0u8, 0, 0, 3]);
        assert_eq!(r.get_key_node_at_idx(0).unwrap().0, max_hash_1_2);
        assert_eq!(r.get_key_node_at_idx(1).unwrap().0, max_hash_3_4);
        assert_eq!(r.get_key_node_at_idx(2).unwrap().0, max_hash);
        assert_eq!(r.ring.len(), 3);

        r.add(vec![0u8, 0, 0, 4]);
        assert_eq!(r.get_key_node_at_idx(0).unwrap().0, max_hash_1_4);
        assert_eq!(r.get_key_node_at_idx(1).unwrap().0, max_hash_1_2);
        assert_eq!(r.get_key_node_at_idx(2).unwrap().0, max_hash_3_4);
        assert_eq!(r.get_key_node_at_idx(3).unwrap().0, max_hash);
        assert_eq!(r.ring.len(), 4);
    }
}
