//! MD5 hash.

use crate::hexadecimal::HexBytes;
use log::{error, trace};
use serde::{de::Error as DeErrorTrait, Deserialize, Deserializer, Serialize};
use std::fmt::{self, Display, Formatter};

const MD5HASH_DE_TARGET: &str = "star_cloudburst::crypto::md5::Md5::deserialize";
const MD5_LEN: usize = 16;

/// MD5 hash wrapper.
///
/// This type wraps one MD5 hash: 128 bits (16 bytes)
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct Md5(HexBytes);

impl From<[u8; MD5_LEN]> for Md5 {
    #[inline]
    fn from(bytes: [u8; MD5_LEN]) -> Self {
        Self(bytes.into())
    }
}

impl Display for Md5 {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        <HexBytes as Display>::fmt(&self.0, f)
    }
}

impl<'de> Deserialize<'de> for Md5 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        trace!(target: MD5HASH_DE_TARGET, "Deserializing an MD5 hash.");

        let bytes = HexBytes::deserialize(deserializer)?;
        let len = bytes.len();

        if len != MD5_LEN {
            error!(
                target: MD5HASH_DE_TARGET,
                "Invalid MD5 hash length: {len} - but should be {MD5_LEN}."
            );

            Err(DeErrorTrait::invalid_length(len, &"16"))
        } else {
            Ok(Md5(bytes))
        }
    }
}
