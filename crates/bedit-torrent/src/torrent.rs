use log::warn;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::HashMap;

use super::{signature::Signature, Info, ParseTorrentError};

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

    /*
    /// Optional torrent validation beyond serialization.
    ///
    /// Torrents may be in an inconsistent state such as missing optional fields that are
    /// required given certain invariants. However, validation may also be too strict because clients
    /// are able to handle somewhat mangled torrents anyway.
    pub fn validate(torrent: &Self) -> Result<(), ParseTorrentError> {
        // unimplemented!();
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
    } */
}
