// pub type CanisterId = u64;
use crate::data::DataBucket;
use crate::index::BigmapIdx;
use serde::{Deserialize, Serialize};
use std::convert::From;

#[derive(Clone, Default, Hash, Serialize, Deserialize, candid::CandidType, Eq, PartialEq)]
#[repr(transparent)]
pub struct CanisterId(pub Vec<u8>);

impl From<DataBucket> for CanisterId {
    fn from(item: DataBucket) -> Self {
        Self {
            0: item.canister_id().0,
        }
    }
}

impl From<u64> for CanisterId {
    fn from(item: u64) -> Self {
        Self {
            // 0: format!("{:x}", item).as_bytes().to_vec(),
            0: item.to_ne_bytes().to_vec(),
        }
    }
}

impl From<BigmapIdx> for CanisterId {
    fn from(item: BigmapIdx) -> Self {
        Self {
            0: item.canister_id().0,
        }
    }
}

impl From<ic_cdk::CanisterId> for CanisterId {
    fn from(item: ic_cdk::CanisterId) -> Self {
        Self { 0: item.0 }
    }
}

impl From<std::vec::Vec<u8>> for CanisterId {
    fn from(item: std::vec::Vec<u8>) -> Self {
        Self { 0: item }
    }
}

impl From<&[u8]> for CanisterId {
    fn from(item: &[u8]) -> Self {
        Self { 0: Vec::from(item) }
    }
}

impl Into<std::vec::Vec<u8>> for CanisterId {
    fn into(self) -> std::vec::Vec<u8> {
        self.0
    }
}

impl std::fmt::Display for CanisterId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let can_id_slice = &self.0;

        // Calculate CRC32 digest of the Canister ID
        let mut crc32hasher = crc32fast::Hasher::new();
        crc32hasher.update(can_id_slice);
        let crc32_bytes = crc32hasher.finalize().to_be_bytes();

        // Append the Canister ID bytes to the calculated CRC32 digest
        let mut crc_and_can_id = Vec::from(crc32_bytes);
        crc_and_can_id.extend(can_id_slice);

        // Base32-encode the concatenated bytes
        let s = data_encoding::BASE32_NOPAD
            .encode(&crc_and_can_id)
            .to_ascii_lowercase();

        // Print with a separator - (dash) inserted every 5 characters.
        let mut s_peekable = s.chars().peekable();
        while s_peekable.peek().is_some() {
            let chunk: String = s_peekable.by_ref().take(5).collect();
            write!(f, "{}", chunk)?;
            if s_peekable.peek().is_some() {
                write!(f, "-")?;
            }
        }
        Ok(())
    }
}

impl std::fmt::Debug for CanisterId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
