use crate::{
    crypto::signature::Signature,
    hexadecimal::HexBytes,
    metainfo::{
        infohash::{InfoHashAny, InfoHashVersioned},
        MetaInfo,
    },
    uri::uriwrapper::UriWrapper,
    uri::Node,
};
use log::debug;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Display, Formatter},
    sync::{OnceLock},
};

// Log target
const TORRENT_TARGET: &str = "star_cloudburst::Torrent::info_hash";

// Based on BEPs as well as:
// https://en.wikipedia.org/wiki/Torrent_file#File_structure
// https://github.com/toby/serde-bencode/blob/master/examples/parse_torrent.rs
// https://wiki.theory.org/BitTorrentSpecification

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
    pub announce: Option<UriWrapper>,
    /// Tiers of announce URLs.
    ///
    /// https://www.bittorrent.org/beps/bep_0012.html
    /// The announce URLs are represented as a list of lists of URLs.
    #[serde(default, rename = "announce-list")]
    pub announce_list: Option<Vec<Vec<UriWrapper>>>,
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
    pub httpseeds: Option<Vec<UriWrapper>>,
    /// Torrent info dictionary.
    ///
    /// The info dict contains integral data on the files shared by the torrent.
    /// This includes suggested names as well as file hashes.
    pub info: MetaInfo,
    /// SHA hash of the torrent's meta info dict.
    #[serde(skip)]
    info_hash_internal: OnceLock<InfoHashAny>,
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
    pub publisher_url: Option<UriWrapper>,
    /// Signatures for signed torrents.
    #[serde(default)]
    pub signatures: Option<HashMap<String, Signature>>,
    /// A non-standard field similar to [Torrent::httpseeds].
    /// https://getright.com/seedtorrent.html
    #[serde(default, rename = "url-list")]
    pub url_list: Option<HashSet<UriWrapper>>,
}

impl Torrent {
    /// Suggested name of the torrent file or directory.
    ///
    /// Example for a single file:
    /// ```rust
    /// use star_cloudburst::{metainfo::{MetaInfo, MetaV1}, Torrent};
    /// use serde_bencode::Error;
    ///
    /// let cats = "d8:announce9:localhost4:infod4:name8:cats.mkv6:pieces20:\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x0012:piece lengthi16eee";
    /// let torrent: Torrent = serde_bencode::from_str(cats)?;
    ///
    /// assert_eq!("cats.mkv", torrent.name());
    /// # Ok::<(), Error>(())
    /// ```
    pub fn name(&self) -> &str {
        match self.info {
            MetaInfo::MetaV1(ref dict) => dict.name.as_str(),
            MetaInfo::MetaV2(ref dict) => dict.name.as_str(),
            MetaInfo::Hybrid(ref dict) => dict.name.as_str(),
        }
    }

    /// Meta info SHA hash.
    /// This is highly subject to change.
    pub fn info_hash(&self) -> Result<InfoHashVersioned<'_>, serde_bencode::Error> {
        // TODO: I don't like that I have to take a mutable reference to Self.
        // I can probably get away with a RefCell since I only need to mutate info_hash once.
        let info_hash = self.info_hash_internal.get_or_try_init(|| {
            // Side effect sin.
            debug!(
                target: TORRENT_TARGET,
                "Info hash doesn't exist on {}. Calculating now.",
                self.name()
            );
            InfoHashAny::calculate_infohash(&self.info)
        })?;

            match self.info {
                MetaInfo::MetaV1(_) => Ok(InfoHashVersioned::V1(&info_hash.sha1)),
                MetaInfo::MetaV2(_) => Ok(InfoHashVersioned::V2(&info_hash.sha2)),
                MetaInfo::Hybrid(_) => Ok(InfoHashVersioned::Hybrid {
                    sha1: &info_hash.sha1,
                    sha2: &info_hash.sha2,
                }),
        }
    }
}

impl Display for Torrent {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct(&format!("Torrent: {}", self.name()))
            .field("Meta info version", &self.info.meta_version_str())
            .field("Files", {
                let files: Vec<_> = self.info.iter_files().collect();
                &format!("{files:#?}")
            })
            .field("Info hash", &self.info_hash())
            .field("Piece length", &self.info.piece_length())
            .finish()
    }
}
