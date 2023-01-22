//! Info hash for meta info dictionaries.
//!
//! [super::MetaV1] is hashed with [SHA1](https://www.bittorrent.org/beps/bep_0003.html).
//! [super::MetaV2] is hashed with [SHA256](https://www.bittorrent.org/beps/bep_0052.html).
//!
//! SHA256 hashes may be truncated to 20 bytes for backwards compatibility or other uses.

use super::MetaInfo;
use crate::crypto::{calculateinfohash::CalculateInfoHash, sha::Sha1, sha2::Sha2};

/// SHA-1 and SHA-2 256 hashes of a torrent's info dict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InfoHashAny {
    pub(crate) sha1: Sha1,
    pub(crate) sha2: Sha2,
}

impl InfoHashAny {
    #[inline]
    pub(crate) fn calculate_infohash(info_dict: &MetaInfo) -> Result<Self, serde_bencode::Error> {
        Ok(Self {
            sha1: Sha1::calculate_infohash(info_dict)?,
            sha2: Sha2::calculate_infohash(info_dict)?,
        })
    }


}

/// Info hash specific to a torrent's info dict version.
/// In other words, a version 1 only torrent will only have a [Sha1] hash.
/// Equality is implemented similarly to [libtorrent](https://libtorrent.org/upgrade_to_2.0-ref.html)
#[derive(Debug, Clone)]
pub enum InfoHashVersioned<'view> {
    V1(&'view Sha1),
    V2(&'view Sha2),
    Hybrid {
        sha1: &'view Sha1,
        sha2: &'view Sha2,
    },
}

impl PartialEq for InfoHashVersioned<'_> {
    fn eq(&self, other: &Self) -> bool {
        // Match whatever variant self is and check that other is the same.
        if let InfoHashVersioned::V1(sha1_self) = self && let InfoHashVersioned::V1(sha1_other) = other {
            sha1_self == sha1_other
        }
        else if let InfoHashVersioned::V2(sha2_self) = self && let InfoHashVersioned::V2(sha2_other) = other {
            sha2_self == sha2_other
        }
        else if let InfoHashVersioned::Hybrid { sha1: sha1_self, sha2: sha2_self } = self && let InfoHashVersioned::Hybrid { sha1: sha1_other, sha2: sha2_other } = other {
            sha1_self == sha1_other && sha2_self == sha2_other
        }
        else {
            false
        }
    }
}
