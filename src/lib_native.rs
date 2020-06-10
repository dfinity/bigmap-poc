// pub type CanisterId = u64;
use crate::data::DataBucket;
use crate::index::BigmapIdx;
use serde::{Deserialize, Serialize};
use std::convert::From;

#[derive(Clone, Debug, Default, Hash, Serialize, Deserialize, candid::CandidType, Eq, PartialEq)]
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

impl Into<std::vec::Vec<u8>> for CanisterId {
    fn into(self) -> std::vec::Vec<u8> {
        self.0
    }
}

impl std::fmt::Display for CanisterId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write!(f, "0x{:?}", self.0.clone())
        write!(f, "ic:{}", hex::encode(self.0.clone()))
    }
}
