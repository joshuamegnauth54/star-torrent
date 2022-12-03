use crate::{
    crypto::{md5::Md5, sha1::Sha1},
    files::fileattributes::TorrentFileAttributes,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::{num::NonZeroU64, path::PathBuf};

/// Files shared by the torrent if multiple as per meta version 1.
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct FlatFiles {
    /// File attribute such as whether the file is executable or hidden.
    #[serde(default)]
    pub attr: Option<TorrentFileAttributes>,
    /// Length of the file in bytes.
    pub length: NonZeroU64,
    /// List of UTF-8 strings consisting of subdirectory names where the last string is the file name.
    pub path: Vec<PathBuf>,
    /// Checksum for the shared file.
    #[serde(default)]
    pub md5sum: Option<Md5>,
    /// SHA1 of file to aid file deduplication.
    #[serde(default)]
    pub sha1: Option<Sha1>,
    /// Paths for symbolic links.
    #[serde(default, rename = "symlink path")]
    pub symlink_path: Option<Vec<String>>,
}
