use log::warn;
use serde::{
    de::{Error as DeError, Unexpected},
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_bencode::value::Value;
use serde_bytes::ByteBuf;
use serde_with::skip_serializing_none;
use std::{collections::HashMap, num::NonZeroU64};

use super::torrent_files::{FileTree, SharedFiles};
use super::torrent_sign::Signature;
use super::ParseTorrentError;

// Based on BEPs as well as:
// https://en.wikipedia.org/wiki/Torrent_file#File_structure
// https://github.com/toby/serde-bencode/blob/master/examples/parse_torrent.rs
// https://wiki.theory.org/BitTorrentSpecification

/// A [`Node`] is (host, port) pair that can be provided through DHT.
///
/// [BEP-0005](https://www.bittorrent.org/beps/bep_0005.html)
///
/// [`Node`]s are not limited to socket addresses but may also be URLs.
#[derive(Debug, Deserialize, Serialize)]
pub struct Node((String, u32));

/// Metainfo on file(s) shared by the torrent.
///
/// The base structure is defined in [BEP-0003](https://www.bittorrent.org/beps/bep_0003.html).
/// Extensions to BEP-0003 are defined in [BEP-0052](https://www.bittorrent.org/beps/bep_0052.html).
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct Info {
    /// Files shared by the torrent.
    #[serde(default)]
    pub files: Option<Vec<SharedFiles>>,
    #[serde(default, rename = "file tree")]
    pub file_tree: Option<FileTree>,
    /// Length of the file in bytes.
    /// This is present if the torrent only shares one file.
    #[serde(default)]
    pub length: Option<u64>,
    /// Shared file's MD5 hash.
    #[serde(default)]
    pub md5sum: Option<String>,
    /// Suggested name of the file or subdirectory to which to save multiple files.
    ///
    /// `name` is a suggestion - files or directories don't have to rigidly follow it.
    pub name: String,
    /// A SHA-1 hash list of each piece concatenated into a string.
    /// The resulting string's length is a multiple of 20 bytes. The position of each hash
    /// corresponds to a file in `files`.
    pub pieces: ByteBuf,
    /// Number of bytes per piece.
    ///
    /// BEP-0003 states that the length is almost always a power of two and usually 2^18.
    #[serde(rename = "piece length")]
    pub piece_length: NonZeroU64,
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
    pub private: Option<bool>,
    /// Merkle tree root hash.
    ///
    /// [BEP-0030](https://www.bittorrent.org/beps/bep_0030.html) adds Merkle trees to reduce torrent file
    /// sizes. Instead of a hash per piece, a Merkle torrent contains the root hash of the tree through which
    /// the hashes of the subseqeuent pieces may be derived.
    #[serde(default, rename = "root hash")]
    pub root_hash: Option<String>,
}

/// Torrent metadata such as the announce urls or DHT [`Node`]s.
///
/// Defined in [BEP-0003](https://www.bittorrent.org/beps/bep_0003.html) and [BEP-0052](https://www.bittorrent.org/beps/bep_0052.html).
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
pub struct Torrent {
    /// Tracker URL.
    ///
    /// `announce` is optional because torrent files may preclude them if `nodes` is present.
    #[serde(default)]
    pub announce: Option<String>,
    /// Tiers of announce URLs.
    ///
    /// https://www.bittorrent.org/beps/bep_0012.html
    /// The announce URLs are represented as a list of lists of URLs.
    #[serde(default, rename = "announce-list")]
    pub announce_list: Option<Vec<Vec<String>>>,
    /// Torrent creator or original uploader.
    #[serde(default, rename = "created by")]
    pub created_by: Option<String>,
    /// Optional comment.
    #[serde(default)]
    pub comment: Option<String>,
    /// Torrent creation date as a Unix timestamp.
    #[serde(default, rename = "creation date")]
    pub creation_date: Option<u64>,
    /// String encoding scheme of `Info::pieces`.
    #[serde(default)]
    pub encoding: Option<String>,
    /// List of web servers that seed the torrent.
    /// https://www.bittorrent.org/beps/bep_0017.html
    #[serde(default)]
    pub httpseeds: Option<Vec<String>>,
    /// Files shared by this torrent.
    pub info: Info,
    /// Torrent file version
    ///
    /// This is specified in BEP-0052 which revises the original torrent format.
    #[serde(default, rename = "meta version")]
    pub meta_version: Option<u8>,
    /// Nodes for distributed hash tables (DHT).
    ///
    /// `nodes` is required for a tracker-less torrent file but optional otherwise.
    #[serde(default)]
    pub nodes: Option<Vec<Node>>,
    /// Torrent publisher's web site.
    #[serde(default, rename = "publisher-url")]
    pub publisher_url: Option<String>,
    /// Signatures for signed torrents.
    #[serde(default)]
    pub signatures: Option<HashMap<String, Signature>>,
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

    /// Deserialize Option<u8> to Option<bool>.
    fn bool_from_int<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
    where
        D: Deserializer<'de>,
    {
        match Value::deserialize(deserializer)? {
            Value::Int(i) => match i {
                0 => Ok(Some(false)),
                1 => Ok(Some(true)),
                nonbool => Err(DeError::invalid_value(
                    Unexpected::Unsigned(nonbool as u64),
                    &"zero or one",
                )),
            },
            Value::Bytes(maybe_none) if maybe_none.is_empty() => Ok(None),
            wrong => {
                let unexpected = match wrong {
                    Value::List(_) => Unexpected::Seq,
                    Value::Dict(_) => Unexpected::Map,
                    Value::Bytes(bytes) if !bytes.is_empty() => {
                        Unexpected::Other("&[u8] that's not empty")
                    }
                    _ => unreachable!("Value::Int and Value::Bytes([]) were checked earlier."),
                };

                Err(DeError::invalid_type(unexpected, &"zero or one"))
            }
        }
    }

    /// Serialize Option<bool> to Option<u8>.
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

#[cfg(test)]
mod tests {
    use super::Torrent;
    use serde::de::{value::Error as DeError, Error as SerdeError, IntoDeserializer};
    use std::error::Error;

    #[test]
    fn bool_from_int_valid() -> Result<(), Box<dyn Error>> {
        let states = [("i0e", false), ("i1e", true)];

        for (value, expected) in states {
            let mut deserializer = serde_bencode::Deserializer::new(value.as_bytes());
            let maybe_bool = Torrent::bool_from_int(&mut deserializer)?
                .ok_or_else(|| DeError::custom("Expected Some({expected})"))?;

            if maybe_bool != expected {
                Err(DeError::custom("Expected {expected}"))?
            }
        }
        Ok(())
    }

    #[test]
    #[should_panic(expected = "Invalid Value: integer `14` (expected: `zero or one`)")]
    fn bool_from_int_invalid() {
        let mut deserializer = serde_bencode::Deserializer::new("i14e".as_bytes());
        Torrent::bool_from_int(&mut deserializer)
            .expect("Invalid Value: integer `14` (expected: `zero or one`)");
    }

    #[test]
    fn bool_from_int_none() {
        let deserializer: serde::de::value::StrDeserializer<'static, DeError> =
            "".into_deserializer();
        // Note to self...doesn't work yet.
        Torrent::bool_from_int(deserializer).unwrap();
    }
}
