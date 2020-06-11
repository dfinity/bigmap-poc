///////////////////////////////////////////////////////////////
// Consistent Hash implementation, based on the HashRing crate
// https://github.com/jeromefroe/hashring-rs/
///////////////////////////////////////////////////////////////
use std::cmp::Ordering;
use std::hash::{BuildHasher, Hash, Hasher};
use wyhash::WyHash;

#[derive(Debug, Default, Clone)]
pub struct WyHashBuilder;

impl BuildHasher for WyHashBuilder {
    type Hasher = WyHash;

    fn build_hasher(&self) -> Self::Hasher {
        WyHash::with_seed(3)
    }
}

// Node is an internal struct used to encapsulate the nodes that will be added
// and removed from `HashRing`
#[derive(Debug, Clone)]
pub(crate) struct Node<T: Clone> {
    pub(crate) key: u64,
    pub(crate) node: T,
}

impl<T: Clone> Node<T> {
    fn new(key: u64, node: T) -> Node<T> {
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
pub struct HashRing<T: Clone, S = WyHashBuilder> {
    hash_builder: S,
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
impl<T: Clone, S> HashRing<T, S> {
    /// Creates an empty `HashRing` which will use the given hash builder.
    pub fn with_hasher(hash_builder: S) -> HashRing<T, S> {
        HashRing {
            hash_builder,
            ring: Vec::new(),
        }
    }

    /// Get the number of nodes in the hash ring.
    pub fn len(&self) -> usize {
        self.ring.len()
    }

    /// Returns true if the ring has no elements.
    pub fn is_empty(&self) -> bool {
        self.ring.len() == 0
    }
}

#[allow(dead_code)]
impl<T: Hash + Clone, S: BuildHasher> HashRing<T, S> {
    /// Add `node` to the hash ring.
    pub fn add(&mut self, node: T) {
        let key = if self.ring.is_empty() {
            u64::max_value()
        } else {
            let mut max_gap_idx = 0;
            let mut max_gap_val = self.ring[max_gap_idx].key;
            for (i, node) in self.ring[1..].iter().enumerate() {
                let i = i + 1; // because we skipped the first element of self.ring
                let gap = node.key - self.ring[i - 1].key;
                if gap > max_gap_val {
                    max_gap_val = gap;
                    max_gap_idx = i;
                }
            }
            if max_gap_idx == 0 {
                self.ring[max_gap_idx].key / 2
            } else {
                self.ring[max_gap_idx - 1].key
                    + (self.ring[max_gap_idx].key - self.ring[max_gap_idx - 1].key) / 2
            }
        };
        // let key = get_key(&self.hash_builder, &node);
        self.ring.push(Node::new(key, node));
        self.ring.sort();
    }

    /// Add `node` to the hash ring, at the `key` position
    pub fn add_with_key(&mut self, key: u64, node: T) {
        self.ring.push(Node::new(key, node));
        self.ring.sort();
    }

    /// Remove `node` from the hash ring. Returns an `Option` that will contain
    /// the `node` if it was in the hash ring or `None` if it was not
    /// present.
    pub fn remove(&mut self, node: &T) -> Option<T> {
        let key = get_key(&self.hash_builder, node);
        match self.ring.binary_search_by(|node| node.key.cmp(&key)) {
            Err(_) => None,
            Ok(n) => Some(self.ring.remove(n).node),
        }
    }

    /// Get the Option<(idx,node)> responsible for `key`.
    /// Returns `None` if the ring is empty
    pub fn get_idx_node_for_key<U: Hash>(&self, key: &U) -> Option<(usize, &T)> {
        if self.ring.is_empty() {
            return None;
        }

        let k = get_key(&self.hash_builder, key);
        let n = match self.ring.binary_search_by(|node| node.key.cmp(&k)) {
            Err(n) => n,
            Ok(n) => n,
        };

        if n == self.ring.len() {
            return Some((0, &self.ring[0].node));
        }

        Some((n, &self.ring[n].node))
    }

    /// Get the Option<(key,node)> responsible for `key`.
    /// Returns `None` if the ring is empty
    pub fn get_key_node<U: Hash>(&self, key: &U) -> Option<(u64, &T)> {
        match self.get_idx_node_for_key(key) {
            Some((idx, node)) => Some((self.ring[idx].key, node)),
            None => None,
        }
    }

    /// Get the node responsible for `key`. Returns an `Option` that will
    /// contain the `node` if the hash ring is not empty or `None` if it was
    /// empty.
    pub fn get<U: Hash>(&self, key: &U) -> Option<&T> {
        match self.get_idx_node_for_key(key) {
            Some((_, node)) => Some(node),
            None => None,
        }
    }

    /// Get the Option<(key,node)> at position `idx`.
    /// Returns `None` if idx is out of bounds
    pub fn get_key_node_at_idx(&self, idx: usize) -> Option<(u64, &T)> {
        if idx >= self.ring.len() {
            return None;
        }
        Some((self.ring[idx].key, &self.ring[idx].node))
    }

    /// Get the Option<node> at position `idx + 1`.
    /// Returns `None` if `idx + 1` is out of bounds
    pub fn get_next_key_node_at_idx(&self, idx: usize) -> Option<(u64, &T)> {
        match self.get_key_node_at_idx(idx + 1) {
            Some(e) => Some(e),
            None => None,
        }
    }

    /// Get the Option<node> at position `idx - 1`.
    /// Returns `None` if `idx - 1` is out of bounds
    pub fn get_prev_key_node_at_idx(&self, idx: usize) -> Option<(u64, &T)> {
        if idx < 1 {
            return None;
        }

        match self.get_key_node_at_idx(idx - 1) {
            Some(e) => Some(e),
            None => None,
        }
    }
}

// An internal function for converting a reference to a `str` into a `u64` which
// can be used as a key in the hash ring.
fn get_key<S, T>(hash_builder: &S, input: T) -> u64
where
    S: BuildHasher,
    T: Hash,
{
    let mut hasher = hash_builder.build_hasher();
    input.hash(&mut hasher);
    let hash = hasher.finish();

    let buf = hash.to_be_bytes();

    u64::from(buf[7]) << 56
        | u64::from(buf[6]) << 48
        | u64::from(buf[5]) << 40
        | u64::from(buf[4]) << 32
        | u64::from(buf[3]) << 24
        | u64::from(buf[2]) << 16
        | u64::from(buf[1]) << 8
        | u64::from(buf[0])
}

#[cfg(test)]
mod tests {
    use super::HashRing;
    // type Entry = Vec<u8>;

    #[test]
    fn hashring_add_key() {
        // Insert key&value pairs and then get the value, and verify the correctness
        let mut r = HashRing::new();
        r.add(vec![0u8, 0, 0, 1]);
        let v = r.get_key_node_at_idx(0);
        assert_eq!(v.unwrap().0, u64::max_value());
        assert_eq!(r.ring.len(), 1);

        r.add(vec![0u8, 0, 0, 2]);
        assert_eq!(r.get_key_node_at_idx(0).unwrap().0, u64::max_value() / 2);
        assert_eq!(r.get_key_node_at_idx(1).unwrap().0, u64::max_value());
        assert_eq!(r.ring.len(), 2);

        let pos12 = u64::max_value() / 2;
        let pos14 = pos12 / 2;
        let pos34 = pos12 + (u64::max_value() - pos12) / 2;
        r.add(vec![0u8, 0, 0, 3]);
        assert_eq!(r.get_key_node_at_idx(0).unwrap().0, u64::max_value() / 2);
        assert_eq!(r.get_key_node_at_idx(1).unwrap().0, pos34);
        assert_eq!(r.get_key_node_at_idx(2).unwrap().0, u64::max_value());
        assert_eq!(r.ring.len(), 3);

        r.add(vec![0u8, 0, 0, 4]);
        assert_eq!(r.get_key_node_at_idx(0).unwrap().0, pos14);
        assert_eq!(r.get_key_node_at_idx(1).unwrap().0, u64::max_value() / 2);
        assert_eq!(r.get_key_node_at_idx(2).unwrap().0, pos34);
        assert_eq!(r.get_key_node_at_idx(3).unwrap().0, u64::max_value());
        assert_eq!(r.ring.len(), 4);
    }
}
