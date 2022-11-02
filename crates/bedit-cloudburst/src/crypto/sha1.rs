//! Types for cryptography used in torrents.

use crate::hexadecimal::HexBytes;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    fmt::{self, Debug, Display},
};

#[derive(Deserialize, Serialize)]
#[serde(transparent)]
pub struct Sha1Hash<'bytes>(Cow<'bytes, HexBytes>);

impl From<[u8; 20]> for Sha1Hash<'_> {
    #[inline]
    fn from(value: [u8; 20]) -> Self {
        let hex: HexBytes = value.to_vec().into();
        Sha1Hash(Cow::Owned(hex))
    }
}

impl Debug for Sha1Hash<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <HexBytes as Debug>::fmt(&self.0, f)
    }
}

impl Display for Sha1Hash<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <HexBytes as Debug>::fmt(&self.0, f)
    }
}
