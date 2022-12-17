use crate::{
    crypto::{md5::Md5, sha1::Sha1},
    files::{FileDisplayInfo, FileTree, FlatFile},
    metainfo::serde_bool_int::{bool_from_int, bool_to_int},
    pieces::{PieceLength, Pieces},
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::num::NonZeroU64;

/// Metainfo on file(s) shared by hybrid torrents.
///
/// Hybrid torrents contain the info dicts for all torrent meta versions.
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct Hybrid {
    /// Files shared by version 1 or hybrid torrents.
    #[serde(default)]
    pub files: Option<Vec<FlatFile>>,
    /// Version 2 or hybrid styled file dictionaries.
    #[serde(default, rename = "file tree")]
    pub file_tree: Option<FileTree>,
    /// Length of the file in bytes.
    /// This is present if the torrent only shares one file.
    #[serde(default)]
    pub length: Option<NonZeroU64>,
    /// Torrent file meta version
    ///
    /// This is specified in BEP-0052 which revises the original torrent format.
    /// Meta version must be greater than or equal to 2. Meta version is increased for
    /// major changes such as deprecating a hash algorithm in favor of a new algo.
    #[serde(default, rename = "meta version")]
    pub meta_version: Option<u8>,
    /// Shared file's MD5 hash.
    #[serde(default)]
    pub md5sum: Option<Md5>,
    /// Suggested name of the file or subdirectory to which to save multiple files.
    ///
    /// `name` is a suggestion - files or directories don't have to rigidly follow it.
    pub name: String,
    /// A SHA-1 hash list of each piece concatenated into a string.
    /// The resulting string's length is a multiple of 20 bytes. The position of each hash
    /// corresponds to a file in `files`.
    pub pieces: Option<Pieces>,
    /// Number of bytes per piece.
    ///
    /// BEP-0003 states that the length is almost always a power of two and usually 2^18.
    #[serde(rename = "piece length")]
    pub piece_length: PieceLength,
    /// Torrent is restricted to private trackers.
    ///
    /// [BEP-0027](https://www.bittorrent.org/beps/bep_0027.html)
    /// Private torrents are only advertised on a private tracker. The swarm is limited to
    /// that particular tracker even if multiple trackers are specified. Torrent clients should
    /// disconnect from all peers if trackers are switched
    #[serde(
        default,
        deserialize_with = "bool_from_int",
        serialize_with = "bool_to_int"
    )]
    pub private: bool,
    /// Merkle tree root hash.
    ///
    /// [BEP-0030](https://www.bittorrent.org/beps/bep_0030.html) adds Merkle trees to reduce torrent file
    /// sizes. Instead of a hash per piece, a Merkle torrent contains the root hash of the tree through which
    /// the hashes of the subseqeuent pieces may be derived.
    #[serde(default, rename = "root hash")]
    pub root_hash: Option<Sha1>,
}
