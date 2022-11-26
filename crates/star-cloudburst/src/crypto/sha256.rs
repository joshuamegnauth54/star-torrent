//! SHA256 hash.

use crate::hexadecimal::HexBytes;
use log::{debug, error};
use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize};
use std::fmt::{self, Display, Formatter};

const SHA256_DE_TARGET: &str = "star_cloudburst::crypto::sha256::Sha256::deserialize";
const SHA256_LEN: usize = 32;

/// SHA256 hash wrapper.
///
/// This wraps one SHA256 hash: 256 bits (32 bytes)
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Sha256(HexBytes);

impl From<[u8; SHA256_LEN]> for Sha256 {
    #[inline]
    fn from(bytes: [u8; SHA256_LEN]) -> Self {
        Self(bytes.into())
    }
}

impl Display for Sha256 {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        <HexBytes as Display>::fmt(&self.0, f)
    }
}

impl<'de> Deserialize<'de> for Sha256 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        debug!(target: SHA256_DE_TARGET, "Deserializing a SHA256 hash.");

        let bytes = HexBytes::deserialize(deserializer)?;
        let len = bytes.len();

        if len != SHA256_LEN {
            error!(
                target: SHA256_DE_TARGET,
                "Invalid SHA256 hash size: {len} - but should be {SHA256_LEN}"
            );

            Err(DeError::invalid_length(len, &"32"))
        } else {
            Ok(Sha256(bytes))
        }
    }
}
