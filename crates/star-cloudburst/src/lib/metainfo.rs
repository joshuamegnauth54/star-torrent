pub mod hybrid;
pub mod infohash;
pub mod metav1;
pub mod metav2;
mod serde_bool_int;

pub use hybrid::Hybrid;
pub use metav1::MetaV1;
pub use metav2::MetaV2;

use crate::{files::filedisplayinfo::{AsFileDisplayInfo, FileDisplayInfoIter}, PieceLength};
use serde::{Deserialize, Serialize};

/// Metainfo on files shared by torrents.
///
/// The base structure is defined in [BEP-0003](https://www.bittorrent.org/beps/bep_0003.html).
/// Version 2.0 extensions to BEP-0003 are defined in [BEP-0052](https://www.bittorrent.org/beps/bep_0052.html).
///
/// More torrent versions may be added in the future so [Info] is non-exhaustive.
#[non_exhaustive]
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MetaInfo {
    /// Meta version 1
    MetaV1(MetaV1),
    /// Meta version 2
    MetaV2(MetaV2),
    /// Backwards compatible amalgamate of all versions.
    Hybrid(Hybrid),
}

impl MetaInfo {
    /// Meta info version agnostic iterator over basic file properties.
    ///
    /// This creates an iterator that yields [crate::files::filedisplayinfo::FileDisplayInfo].
    pub fn iter_files(&self) -> FileDisplayInfoIter {
        match self {
            MetaInfo::MetaV1(info) => {
                let branches = info.as_file_display();
                FileDisplayInfoIter { branches }
            }
            MetaInfo::MetaV2(info) => {
                let branches = info.file_tree.as_file_display();
                FileDisplayInfoIter { branches }
            }
            MetaInfo::Hybrid(info) => {
                if let Some(tree) = &info.file_tree {
                    let branches = tree.as_file_display();
                    FileDisplayInfoIter { branches }
                } else {
                    panic!("Hybrid torrent doesn't have `FileTree`.\nFIX THIS LATER.");
                }
            }
        }
    }

    /// Meta info version as a str.
    #[inline]
    pub fn meta_version_str(&self) -> &str {
        match self {
            MetaInfo::MetaV1(_) => "1",
            MetaInfo::MetaV2(_) => "2",
            MetaInfo::Hybrid(_) => "hybrid",
        }
    }

    #[inline]
    pub fn piece_length(&self) -> PieceLength {
        match self {
            MetaInfo::MetaV1(info) => info.piece_length,
            MetaInfo::MetaV2(info) => info.piece_length,
            MetaInfo::Hybrid(info) => info.piece_length
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_test::{assert_tokens, Token};
    #[test]
    fn info_metav1_only() {}

    #[test]
    fn info_metav2_only() {}

    #[test]
    fn info_hybrid() {}
}
