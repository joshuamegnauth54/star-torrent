use crate::hexadecimal::HexBytes;

use super::{signature::Signature, urlwrapper::UrlWrapper, Info};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::{HashMap, HashSet};

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
/// Types are validated during parsing when possible so that invalid states are impossible. Fields that aren't declared below are
/// ignored when built with `--release`.
/// Defined in [BEP-0003](https://www.bittorrent.org/beps/bep_0003.html) and [BEP-0052](https://www.bittorrent.org/beps/bep_0052.html).
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
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
    /// Piece layers for Merkel tree (meta version 2).
    /// https://www.bittorrent.org/beps/bep_0052.html
    #[serde(default, rename = "piece layers")]
    pub piece_layers: Option<HashMap<HexBytes, HexBytes>>,
    /// Torrent publisher's web site.
    #[serde(default, rename = "publisher-url")]
    pub publisher_url: Option<UrlWrapper>,
    /// Signatures for signed torrents.
    #[serde(default)]
    pub signatures: Option<HashMap<String, Signature>>,
    /// A non-standard field similar to [Torrent::httpseeds].
    /// https://getright.com/seedtorrent.html
    #[serde(default, rename = "url-list")]
    pub url_list: Option<HashSet<UrlWrapper>>,
}

impl Torrent {
    /// Suggested name of the torrent file or directory.
    ///
    /// ```rust
    /// use bedit_cloudburst::{Info, MetaV1, Torrent};
    /// use serde::de::value::Error;
    ///
    /// let cats = "d8:announce9:localhost4:info:d4info4:name8:cats.mkv6:pieces:\x00\x00\x00\x00\x00\x00\x00\x00\x00\x0011:piece length:i16eee";
    /// let torrent: Torrent = serde_bencode::from_str(cats)?;
    ///
    /// assert_eq!("cats.mkv", torrent.name());
    /// # Ok::<(), Error>(())
    /// ```
    pub fn name(&self) -> &str {
        match self.info {
            Info::MetaV1(ref dict) => dict.name.as_str(),
            Info::MetaV2(ref dict) => dict.name.as_str(),
            Info::Hybrid(ref dict) => dict.name.as_str(),
        }
    }
}
