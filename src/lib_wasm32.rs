// pub type CanisterId = Vec<u8>;
use crate::data::DataBucket;
use crate::index::BigmapIdx;
use serde::{Deserialize, Serialize};
use std::convert::From;

#[derive(Clone, Debug, Default, Hash, Serialize, Deserialize, candid::CandidType)]
#[repr(transparent)]
pub struct CanisterId(pub Vec<u8>);

impl From<DataBucket> for CanisterId {
    fn from(item: DataBucket) -> Self {
        Self {
            0: item.canister_id().0,
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

impl Into<ic_cdk::CanisterId> for CanisterId {
    fn into(self) -> ic_cdk::CanisterId {
        ic_cdk::CanisterId::from_str_unchecked(std::str::from_utf8(&self.0).unwrap())
            .expect("Could not parse the CanisterId")
    }
}

impl std::fmt::Display for CanisterId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let can_id = self.0.clone();
        let mut crc8 = crc8::Crc8::create_msb(7);
        let crc = crc8.calc(&can_id, can_id.len() as i32, 0);
        write!(f, "ic:{}{:02X}", hex::encode_upper(self.0.clone()), crc)
    }
}
