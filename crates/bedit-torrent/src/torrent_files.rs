use either::Either;
use serde::{
    Deserialize, Serialize,
};
use std::{borrow::Cow, collections::BTreeMap, num::NonZeroU64};

use super::fileattributes::TorrentFileAttributes;

/// Files shared by the torrent if multiple.
#[derive(Debug, Deserialize, Serialize)]
pub struct SharedFiles {
    /// File attribute such as whether the file is executable or hidden.
    #[serde(default)]
    pub attr: Option<TorrentFileAttributes>,
    /// Length of the file in bytes.
    length: NonZeroU64,
    /// List of UTF-8 strings consisting of subdirectory names where the last string is the file name.
    path: Vec<String>,
    /// Checksum for the shared file.
    #[serde(default)]
    md5sum: Option<String>,
    /// SHA1 of file to aid file deduplication.
    #[serde(default)]
    sha1: Option<String>,
    #[serde(default, rename = "symlink path")]
    symlink_path: Option<Vec<String>>,
}

/// An empty str.
#[repr(transparent)]
#[derive(Default, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[serde(transparent)]
pub struct EmptyString(Cow<'static, str>);

/// File info for version 2.0 torrents.
///
/// V2 torrents use a different encoding scheme for files. Files and directories are stored as a tree where
/// the leaf nodes describe files.
#[derive(Debug, Deserialize, Serialize)]
pub struct FileTreeInfo {
    /// File attribute such as whether a file is executable or hidden.
    #[serde(default)]
    attr: Option<TorrentFileAttributes>,
    /// Length of the file in bytes.
    length: NonZeroU64,
    /// Merkel tree root.
    #[serde(default, rename = "pieces root")]
    pieces_root: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TreeNode {
    #[serde(with = "either::serde_untagged")]
    pub node: Either<BTreeMap<EmptyString, FileTreeInfo>, BTreeMap<String, TreeNode>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct FileTree {
    root: TreeNode,
}
