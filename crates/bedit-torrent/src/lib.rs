//! `bedit-torrent` provides strongly typed data structures for serializing and deserializing torrents.

mod fileattributes;
mod files;
mod info;
mod signature;
mod torrent;

pub use fileattributes::{FileAttribute, TorrentFileAttributes};
pub use files::{FileTree, FileTreeEntry, FileTreeInfo, SharedFiles};
pub use info::{Hybrid, Info, MetaV1, MetaV2};
pub use signature::{SignInfo, Signature};
pub use torrent::{Node, Torrent};
