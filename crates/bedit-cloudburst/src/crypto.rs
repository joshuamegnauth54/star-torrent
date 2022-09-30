//! Types for cryptography used in torrents.

use arrayvec::ArrayVec;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug, Display};

#[derive(Deserialize, Serialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Sha1Hash(ArrayVec<u8, 20>);

impl From<[u8; 20]> for Sha1Hash {
    #[inline]
    fn from(value: [u8; 20]) -> Self {
        Sha1Hash(ArrayVec::from(value))
    }
}

impl Debug for Sha1Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{:#02x?}", self.0)
        } else {
            write!(f, "{:02x?}", self.0)
        }
    }
}

impl Display for Sha1Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(&self.0))
    }
}
