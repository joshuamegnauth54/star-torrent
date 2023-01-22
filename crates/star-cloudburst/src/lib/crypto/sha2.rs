//! SHA256 hash.

use super::calculateinfohash::CalculateInfoHash;
use crate::{hexadecimal::HexBytes, metainfo::MetaInfo};
use digest::{
    consts,
    core_api::{CoreWrapper, CtVariableCoreWrapper},
};
use log::{error, trace};
use serde::{de::Error as DeErrorTrait, Deserialize, Deserializer, Serialize};
use std::fmt::{self, Display, Formatter};

const SHA256_DE_TARGET: &str = "star_cloudburst::crypto::sha256::Sha256::deserialize";
const SHA256_LEN: usize = 32;

/// SHA256 hash wrapper.
///
/// This wraps one SHA256 hash: 256 bits (32 bytes)
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Sha2(HexBytes);

impl From<[u8; SHA256_LEN]> for Sha2 {
    #[inline]
    fn from(bytes: [u8; SHA256_LEN]) -> Self {
        Self(bytes.into())
    }
}

impl CalculateInfoHash<SHA256_LEN> for Sha2 {
    type Error = serde_bencode::Error;
    type Info = MetaInfo;
    type Hasher = CoreWrapper<CtVariableCoreWrapper<sha2::Sha256VarCore, consts::U32>>;
}

impl Display for Sha2 {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        <HexBytes as Display>::fmt(&self.0, f)
    }
}

impl<'de> Deserialize<'de> for Sha2 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        trace!(target: SHA256_DE_TARGET, "Deserializing a SHA256 hash.");

        let bytes = HexBytes::deserialize(deserializer)?;
        let len = bytes.len();

        if len != SHA256_LEN {
            error!(
                target: SHA256_DE_TARGET,
                "Invalid SHA256 hash size: {len} - but should be {SHA256_LEN}"
            );

            Err(DeErrorTrait::invalid_length(len, &"32"))
        } else {
            Ok(Sha2(bytes))
        }
    }
}
