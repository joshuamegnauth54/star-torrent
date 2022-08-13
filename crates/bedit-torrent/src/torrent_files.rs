use either::Either;
use serde::{
    de::{value::Error as DeError, Error as DeErrorTrait},
    Deserialize, Serialize,
};
use std::{borrow::Cow, collections::BTreeMap, num::NonZeroU64};

/// File attributes.
///
/// Executable = 'x'
/// Hidden = 'h'
/// Padding = 'p'
/// Symlink = 'l'
///
/// Extended file properties are defined in [BEP-0047](https://www.bittorrent.org/beps/bep_0047.html).
#[derive(Clone, Copy, Debug)]
pub enum FileAttribute {
    Executable,
    Hidden,
    Padding,
    Symlink,
}

impl TryFrom<char> for FileAttribute {
    type Error = DeError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'x' => Ok(Self::Executable),
            'h' => Ok(Self::Hidden),
            'p' => Ok(Self::Padding),
            'l' => Ok(Self::Symlink),
            _ => Err(DeError::unknown_variant(
                &value.to_string(),
                &["x", "h", "p", "l"],
            )),
        }
    }
}

impl TryFrom<&str> for FileAttribute {
    type Error = DeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "x" => Ok(Self::Executable),
            "h" => Ok(Self::Hidden),
            "p" => Ok(Self::Padding),
            "l" => Ok(Self::Symlink),
            _ => Err(DeError::unknown_variant(value, &["x", "h", "p", "l"])),
        }
    }
}

impl From<FileAttribute> for &str {
    fn from(other: FileAttribute) -> Self {
        match other {
            FileAttribute::Executable => "x",
            FileAttribute::Hidden => "h",
            FileAttribute::Padding => "p",
            FileAttribute::Symlink => "l",
        }
    }
}

impl Serialize for FileAttribute {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str((*self).into())
    }
}

impl<'de> Deserialize<'de> for FileAttribute {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let maybe_attr = char::deserialize(deserializer)?;
        maybe_attr
            .try_into()
            // To do: Can't constraint the type of D::Error to DeError yet.
            .map_err(|_| D::Error::unknown_variant(&maybe_attr.to_string(), &["x", "h", "p", "l"]))
    }
}

#[derive(Debug, Clone)]
pub struct TorrentFileAttributes(Vec<FileAttribute>);

impl<'de> Deserialize<'de> for TorrentFileAttributes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de> {
        let attr = str::deserialize(deserializer)?;
        let attrs_parsed: Result<Vec<_>, _> = attr.iter()
    }
}

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
    attr: Option<TorrentFileAttribute>,
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
