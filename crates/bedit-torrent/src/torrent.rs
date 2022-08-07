use either::Either;
use log::warn;
use serde::{
    de::{Error as DeError, Unexpected},
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_bytes::ByteBuf;
use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap},
    num::NonZeroU64,
};

use super::ParseTorrentError;

// Based on BEPs as well as:
// https://en.wikipedia.org/wiki/Torrent_file#File_structure
// https://github.com/toby/serde-bencode/blob/master/examples/parse_torrent.rs
// https://wiki.theory.org/BitTorrentSpecification

/// A node is (host, port) pair that can be provided through DHT.
///
/// [BEP-0005](https://www.bittorrent.org/beps/bep_0005.html)
///
/// Nodes are not limited to socket addresses but may also be URLs.
#[derive(Debug, Deserialize, Serialize)]
pub struct Node((String, u32));

/// Files shared by the torrent if multiple.
#[derive(Debug, Deserialize, Serialize)]
pub struct SharedFiles {
    length: u64,
    /// List of UTF-8 strings consisting of subdirectory names where the last string is the file name.
    path: Vec<String>,
    /// Checksum for the shared file.
    #[serde(default)]
    md5sum: Option<String>,
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
    /// Length of the file in bytes.
    length: u64,
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

/// Metainfo on file(s) shared by the torrent.
///
/// The base structure is defined in [BEP-0003](https://www.bittorrent.org/beps/bep_0003.html).
/// Extensions to BEP-0003 are defined in [BEP-0052](https://www.bittorrent.org/beps/bep_0052.html).
#[derive(Debug, Deserialize, Serialize)]
pub struct Info {
    /// Files shared by the torrent.
    #[serde(default)]
    files: Option<Vec<SharedFiles>>,
    #[serde(default, rename = "file tree")]
    file_tree: Option<FileTree>,
    /// Length of the file in bytes.
    /// This is present if the torrent only shares one file.
    #[serde(default)]
    length: Option<u64>,
    /// Shared file's MD5 hash.
    #[serde(default)]
    md5sum: Option<String>,
    /// Suggested name of the file or subdirectory to which to save multiple files.
    ///
    /// `name` is a suggestion - files or directories don't have to rigidly follow it.
    name: String,
    /// A SHA-1 hash list of each piece concatenated into a string.
    /// The resulting string's length is a multiple of 20 bytes. The position of each hash
    /// corresponds to a file in `files`.
    pieces: ByteBuf,
    /// Number of bytes per piece.
    ///
    /// BEP-0003 states that the length is almost always a power of two and usually 2^18.
    #[serde(rename = "piece length")]
    piece_length: NonZeroU64,
    /// Torrent is restricted to private trackers.
    ///
    /// [BEP-0027](https://www.bittorrent.org/beps/bep_0027.html)
    /// Private torrents are only advertised on a private tracker. The swarm is limited to
    /// that particular tracker even if multiple trackers are specified. Torrent clients should
    /// disconnect from all peers if trackers are switched
    #[serde(
        default,
        deserialize_with = "Torrent::bool_from_int",
        serialize_with = "Torrent::bool_to_int"
    )]
    private: Option<bool>,
    /// Merkle tree root hash.
    ///
    /// [BEP-0030](https://www.bittorrent.org/beps/bep_0030.html) adds Merkle trees to reduce torrent file
    /// sizes. Instead of a hash per piece, a Merkle torrent contains the root hash of the tree through which
    /// the hashes of the subseqeuent pieces may be derived.
    #[serde(default, rename = "root hash")]
    root_hash: Option<String>,
}

/// Additional info for `Signature`; unused.
#[derive(Debug, Deserialize, Serialize)]
pub struct SignInfo {}

/// Signatures for signed torrents. [BEP-0035](https://www.bittorrent.org/beps/bep_0035.html)
#[derive(Debug, Deserialize, Serialize)]
pub struct Signature {
    /// X.509 certificate used to sign the torrent. The user should have a certificate elsewhere if this is missing.
    #[serde(default)]
    certificate: Option<String>,
    /// Extension info (currently unspecified)
    #[serde(default)]
    info: Option<SignInfo>,
    /// Signature of torrent's `Info` and `Signature`'s `SignInfo` if present.
    signature: String,
}

/// Torrent metadata
///
/// Defined in [BEP-0003](https://www.bittorrent.org/beps/bep_0003.html) and [BEP-0052](https://www.bittorrent.org/beps/bep_0052.html).
#[derive(Debug, Deserialize, Serialize)]
pub struct Torrent {
    /// Tracker URL.
    ///
    /// `announce` is optional because torrent files may preclude them if `nodes` is present.
    #[serde(default)]
    announce: Option<String>,
    /// Tiers of announce URLs.
    ///
    /// https://www.bittorrent.org/beps/bep_0012.html
    /// The announce URLs are represented as a list of lists of URLs.
    #[serde(default, rename = "announce-list")]
    announce_list: Option<Vec<Vec<String>>>,
    /// Torrent creator or original uploader.
    #[serde(default, rename = "created by")]
    created_by: Option<String>,
    /// Optional comment.
    #[serde(default)]
    comment: Option<String>,
    /// Torrent creation date as a Unix timestamp.
    #[serde(default, rename = "creation date")]
    creation_date: Option<u64>,
    /// String encoding scheme of `Info::pieces`.
    #[serde(default)]
    encoding: Option<String>,
    /// List of web servers that seed the torrent.
    /// https://www.bittorrent.org/beps/bep_0017.html
    #[serde(default)]
    httpseeds: Option<Vec<String>>,
    /// Files shared by this torrent.
    info: Info,
    /// Torrent file version
    ///
    /// This is specified in BEP-0052 which revises the original torrent format.
    #[serde(default, rename = "meta version")]
    meta_version: Option<u8>,
    /// Nodes for distributed hash tables (DHT).
    ///
    /// `nodes` is required for a tracker-less torrent file but optional otherwise.
    #[serde(default)]
    nodes: Option<Vec<Node>>,
    /// Torrent publisher's web site.
    #[serde(default, rename = "publisher-url")]
    publisher_url: Option<String>,
    /// Signatures for signed torrents.
    #[serde(default)]
    signatures: HashMap<String, Signature>,
}

impl Torrent {
    #[inline]
    pub fn de_from_str(torrent: &str) -> Result<Self, ParseTorrentError> {
        serde_bencode::from_str(torrent).map_err(Into::into)
    }

    #[inline]
    pub fn de_from_bytes(torrent: &[u8]) -> Result<Self, ParseTorrentError> {
        serde_bencode::from_bytes(torrent).map_err(Into::into)
    }

    #[inline]
    pub fn se_to_string(&self) -> Result<String, ParseTorrentError> {
        serde_bencode::to_string(self).map_err(Into::into)
    }

    #[inline]
    pub fn se_to_bytes(&self) -> Result<Vec<u8>, ParseTorrentError> {
        serde_bencode::to_bytes(self).map_err(Into::into)
    }

    /// Optional torrent validation beyond serialization.
    ///
    /// Torrents may be in an inconsistent state such as missing optional fields that are
    /// required given certain invariants. However, validation may also be too strict because clients
    /// are able to handle somewhat mangled torrents anyway.
    pub fn validate(torrent: &Self) -> Result<(), ParseTorrentError> {
        //unimplemented!();
        // Validation errors for version 2.
        if let Some(version) = torrent.meta_version {
            if version < 2 {
                return Err(ParseTorrentError::InvalidVersion(version));
            }

            // Piece length should be => 16 and a power of two.
            let piece_length = torrent.info.piece_length;
            if !piece_length.is_power_of_two() {
                warn!("Field 'piece length' should be a power of two. Got: {piece_length}.")
            }
            if piece_length < 16.try_into().unwrap() {
                return Err(ParseTorrentError::PieceLength(piece_length));
            }

            Ok(())
        } else {
            match (
                torrent.info.length.is_some(),
                torrent.info.files.is_some(),
                torrent.info.file_tree.is_some(),
            ) {
                (true, true, false) => Err(ParseTorrentError::AmbiguousFiles("length and files")),
                (true, false, true) => {
                    Err(ParseTorrentError::AmbiguousFiles("length and file_tree"))
                }
                (false, true, true) => {
                    Err(ParseTorrentError::AmbiguousFiles("files and file tree"))
                }
                (false, false, false) => Err(ParseTorrentError::AmbiguousFiles("no files")),
                (true, true, true) => Err(ParseTorrentError::AmbiguousFiles(
                    "length, files, and file_tree",
                )),
                // Remaining states are valid.
                _ => Ok(()),
            }
        }
    }

    /// Convert Option::<u8> to Option::<bool>.
    fn bool_from_int<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
    where
        D: Deserializer<'de>,
    {
        match Option::<u8>::deserialize(deserializer)? {
            Some(0) => Ok(Some(false)),
            Some(1) => Ok(Some(true)),
            None => Ok(None),
            nonbool => Err(DeError::invalid_value(
                Unexpected::Unsigned(nonbool.unwrap_or_default() as u64),
                &"zero or one",
            )),
        }
    }

    fn bool_to_int<S>(private: &Option<bool>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match private {
            Some(true) => serializer.serialize_some(&1),
            Some(false) => serializer.serialize_some(&0),
            None => serializer.serialize_none(),
        }
    }
}
