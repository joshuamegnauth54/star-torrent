//! Types for cryptography used in torrents.

use super::hex::HexBytes;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Display};

#[derive(Deserialize, Serialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Sha1Hash(HexBytes<'static>);

impl From<[u8; 20]> for Sha1Hash {
    #[inline]
    fn from(value: [u8; 20]) -> Self {
        Sha1Hash(value.to_vec().into())
    }
}

impl Debug for Sha1Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <HexBytes as Debug>::fmt(&self.0, f)
    }
}

impl Display for Sha1Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <HexBytes as Debug>::fmt(&self.0, f)
    }
}
