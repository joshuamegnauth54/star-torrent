//! Types representing files shared by a torrent.
//!
//! [flatfiles::FlatFiles] is for version 1 or hybrid torrents.
//! [filetree::FileTree] is for version 2 or hybrid torrents.
//!
//! Compared to version 1 torrents, version 2 torrents may be smaller in size due to [filetree::FileTree]s deduplicating paths.

pub mod fileattributes;
pub mod filedisplayinfo;
pub mod filetree;
pub mod flatfile;

pub use fileattributes::{FileAttribute, TorrentFileAttributes};
pub use filedisplayinfo::FileDisplayInfo;
pub use filetree::{
    FileTree, FileTreeDepthFirstIter, FileTreeEntry, FileTreeInfo, FileTreePathView,
};
pub use flatfile::{FlatFile, MetaV1FileRepr};
