//! Types representing files shared by a torrent.

use either::Either;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    num::NonZeroU64,
};

use super::fileattributes::TorrentFileAttributes;

/// Files shared by the torrent if multiple.
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct SharedFiles {
    /// File attribute such as whether the file is executable or hidden.
    #[serde(default)]
    pub attr: Option<TorrentFileAttributes>,
    /// Length of the file in bytes.
    pub length: NonZeroU64,
    /// List of UTF-8 strings consisting of subdirectory names where the last string is the file name.
    pub path: Vec<String>,
    /// Checksum for the shared file.
    #[serde(default)]
    pub md5sum: Option<String>,
    /// SHA1 of file to aid file deduplication.
    #[serde(default)]
    pub sha1: Option<String>,
    /// Paths for symbolic links.
    #[serde(default, rename = "symlink path")]
    pub symlink_path: Option<Vec<String>>,
}

/// An empty str.
#[repr(transparent)]
#[derive(Default, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
#[serde(transparent)]
pub struct EmptyString(Cow<'static, str>);

/// File info for version 2.0 torrents.
///
/// V2 torrents use a different encoding scheme for files. Files and directories are stored as a tree where
/// the leaf nodes describe files.
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct FileTreeInfo {
    /// File attribute such as whether a file is executable or hidden.
    #[serde(default)]
    pub attr: Option<TorrentFileAttributes>,
    /// Length of the file in bytes.
    pub length: NonZeroU64,
    /// Merkel tree root.
    #[serde(default, rename = "pieces root")]
    pub pieces_root: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct FileTree {
    pub node: HashMap<String, Either<FileTreeInfo, FileTree>>,
}
