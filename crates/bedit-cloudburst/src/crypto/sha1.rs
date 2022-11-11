//! SHA-1 hash.

use crate::hexadecimal::HexBytes;
use log::{debug, error};
use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize};
use std::fmt::{self, Display, Formatter};

const SHA1HASH_DE_TARGET: &str = "bedit_cloudburst::crypto::sha1::Sha1::deserialize";
const SHA1_LEN: usize = 20;

/// SHA1 hash wrapper.
///
/// This type wraps one SHA1 hash: 160 bits (20 bytes).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct Sha1(HexBytes);

impl From<[u8; SHA1_LEN]> for Sha1 {
    #[inline]
    fn from(value: [u8; SHA1_LEN]) -> Self {
        Self(value.into())
    }
}

/*
impl Debug for Sha1 {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <HexBytes as Debug>::fmt(&self.0, f)
    }
}
*/

impl Display for Sha1 {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        <HexBytes as Display>::fmt(&self.0, f)
    }
}

impl<'de> Deserialize<'de> for Sha1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        debug!(target: SHA1HASH_DE_TARGET, "Deserializing a SHA1 hash.");

        let bytes = HexBytes::deserialize(deserializer)?;
        let len = bytes.len();

        if len != SHA1_LEN {
            error!(
                target: SHA1HASH_DE_TARGET,
                "Invalid SHA1 hash length: {len} - but should be {SHA1_LEN}."
            );
            Err(DeError::invalid_length(len, &"20"))
        } else {
            Ok(Sha1(bytes))
        }
    }
}
