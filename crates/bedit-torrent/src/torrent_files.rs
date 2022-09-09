//! Types representing files shared by a torrent.

//use either::Either;
use itertools::{Either, Itertools};
use log::{debug, error};
use serde::{
    de::{value::Error as DeError, Error as DeErrorTrait, Unexpected},
    Deserialize, Serialize,
};
use serde_bencode::value::Value;
use serde_bytes::ByteBuf;
use serde_with::skip_serializing_none;
use std::{
    borrow::Cow,
    collections::{BTreeMap, HashMap, VecDeque},
    num::NonZeroU64,
};

use crate::error::{value_to_unexpected, ParseTorrentError};

use super::fileattributes::TorrentFileAttributes;

/// Files shared by the torrent if multiple.
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SharedFiles {
    /// File attribute such as whether the file is executable or hidden.
    #[serde(default)]
    pub attr: Option<TorrentFileAttributes>,
    /// Length of the file in bytes.
    pub length: NonZeroU64,
    /// List of UTF-8 strings consisting of subdirectory names where the last string is the file name.
    pub path: Vec<String>,
    /// Checksum for the shared file.
    #[serde(default)]
    pub md5sum: Option<String>,
    /// SHA1 of file to aid file deduplication.
    #[serde(default)]
    pub sha1: Option<String>,
    /// Paths for symbolic links.
    #[serde(default, rename = "symlink path")]
    pub symlink_path: Option<Vec<String>>,
}

/// An empty str.
#[repr(transparent)]
#[derive(Default, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
#[serde(transparent)]
pub struct EmptyString(Cow<'static, str>);

/// File info for version 2.0 torrents.
///
/// V2 torrents use a different encoding scheme for files. Files and directories are stored as a tree where
/// the leaf nodes describe files.
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FileTreeInfo {
    /// File attribute such as whether a file is executable or hidden.
    #[serde(default)]
    pub attr: Option<TorrentFileAttributes>,
    /// Length of the file in bytes.
    pub length: NonZeroU64,
    /// Merkel tree root.
    #[serde(default, rename = "pieces root")]
    pub pieces_root: Option<ByteBuf>,
}

impl TryFrom<HashMap<String, Value>> for FileTreeInfo {
    type Error = DeError;

    fn try_from(value: HashMap<String, Value>) -> Result<Self, Self::Error> {
        // I DON'T want to ignore any errors. I could turn the Result into an Option such that the error is None,
        // but I'd rather indicate that a torrent is invalid.
        let attr = value.remove("attr").map(|value| {
            if let Value::Bytes(bytes) = value {
                let attr_str = String::from_utf8_lossy(&bytes);
                TorrentFileAttributes::try_from(attr_str.as_ref())
            } else {
                Err(DeError::invalid_type(
                    value_to_unexpected(&value),
                    &"string of file attributes",
                ))
            }
        });

        // I want to destructure the Result without using ok_or_else to transform the Option.
        let attr = match attr {
            Some(maybe_attrs) => Some(maybe_attrs?),
            None => None,
        };

        // Length is the only required field
        let length = value
            .remove("length")
            .map(|value| {
                if let Value::Int(length) = value {
                    Ok(length)
                } else {
                    Err(DeError::invalid_type(
                        value_to_unexpected(&value),
                        &"unsigned integer",
                    ))
                }
            })
            .ok_or_else(|| DeError::missing_field("length"))?
            .and_then(|int| {
                // I don't want to shadow int because then I'll have to cast the u64 to an i64 again.
                let int_cast: u64 = int.try_into().map_err(|_| {
                    DeError::invalid_value(
                        Unexpected::Signed(int),
                        &"non-zero and positive integer",
                    )
                })?;

                NonZeroU64::try_from(int_cast).map_err(|_| {
                    DeError::invalid_value(Unexpected::Signed(int), &"non-zero integer")
                })
            })?;

        let pieces_root = value.remove("pieces_root").map(|value| {
            if let Value::Bytes(root) = value {
                Ok(root)
            } else {
                Err(DeError::invalid_type(
                    value_to_unexpected(&value),
                    &"pieces root as a string",
                ))
            }
        });

        let pieces_root = match pieces_root {
            Some(bytes) => {
                let bytes = bytes?;
                Some(ByteBuf::from(bytes))
            }
            None => None,
        };

        #[cfg(debug_assertions)]
        for (key, value) in value {
            debug!("[unknown fields] (FileTreeInfo from HashMap) key: {key}, value: {value:#?}");
        }

        Ok(Self {
            attr,
            length,
            pieces_root,
        })
    }
}

#[derive(Debug)]
#[repr(transparent)]
struct FileTreeEntry(pub Either<FileTreeInfo, FileTree>);

struct FileTreeEntryTemp(Either<FileTreeInfo, HashMap<String, Value>>);

impl TryFrom<HashMap<String, Value>> for FileTreeEntryTemp {
    type Error = ParseTorrentError;

    fn try_from(value: HashMap<String, Value>) -> Result<Self, Self::Error> {
        match value.iter().map(|(name, _)| name.as_str()).next() {
            // Files are keyed with an empty string in the nested dict.
            Some("") => {
                // A file should only have one key of "". Therefore anything else is invalid.
                if value.len() == 1 {
                    let file_dict: FileTreeInfo = value.try_into()?;
                    Ok(FileTreeEntryTemp(Either::Left(file_dict)))
                } else {
                    debug!("Invalid HashMap for FileTreeInfo: {value:#?}");
                    Err(DeError::invalid_length(
                        value.len(),
                        &"map with an empty string as its only key",
                    ))?
                }
            }
            _ => Ok(FileTreeEntryTemp(Either::Right(value))),
        }
    }
}

impl<'de> Deserialize<'de> for FileTree {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Deserialize the nested bencoded dict.
        // "file tree" is a HashMap<String, HashMap<String, file | dir>>
        // FileTreeEntry wraps the value into an Either<FileTreeInfo, FileTree>
        let mut dirs = VecDeque::new();
        dirs.push_front(HashMap::<String, Value>::deserialize(deserializer)?);
        let mut tree = FileTree {
            node: BTreeMap::default(),
        };

        for (mut name, node) in dirs.pop_front() {
            let entry = node.try_into()?;

            match entry.0 {
                Either::Left(file) => {
                    while tree.node.contains_key(&name) {
                        error!("[deserialize] Torrent has duplicate file: {name}; renaming.");
                        name += "_dup";
                    }
                    tree.node.insert(name, file);
                }
                Either::Right(dir) => {
                    while tree.node.contains_key(&name) {
                        error!("[deserialize] Torrent has duplicate directory: {name}; renaming.");
                        name += "_dup";
                    }
                    dirs.push_front(dir);
                }
            }
        }

        Ok(tree)
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct FileTree {
    pub node: BTreeMap<String, FileTreeEntry>,
}

struct FileTreePaths(HashMap<Vec<u8>, Value>);

impl FileTreePaths {
    #[inline]
    fn depth_first_map(self) -> impl Iterator<Item = (Vec<u8>, Value)> {
        self.0.into_iter()
    }
}

struct FileTreePathsDFS<I>
where
    I: Iterator<Item = (Vec<u8>, Value)>,
{
    iterator: I,
}

/*
impl<I> Iterator for FileTreePathsDFS<I>
where
    I: Iterator<Item = (Vec<u8>, Value)>,
{
    type Item = Result<(String, FileTreeEntry), ParseTorrentError>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((bytes, value)) = self.iterator.next() {
            // The structure is something like:
            // dir1 {
            //     (directories | files)...
            // },
            // dir2 {
            //     (directories | files)...
            // },
            // file1.txt {
            //     "": InfoMap
            // },
            // ad nauseam...
            //
            // So...the type is Dict<String, Value::Dict<String, Dir | FileMap>>
            // The implementation needs to check if the dictionary value is a file or a directory,
            // and the recurse or return as appropriate.

            let empty_bytes = "".as_bytes();
            let name = String::from_utf8_lossy(&bytes);

            if let Value::Dict(value) = value {
                match value.iter().map(|(name, value)| name).next() {
                    empty_bytes => {
                        let file_dict: FileTreeInfo = value.try_into()?;
                        Ok(Some(name, file_dict))
                    }
                    dir => {
                        let paths_iter = FileTreePaths(dir_dict);
                        Some(paths_iter.depth_first_map().collect())
                    }
                }
            } else {
                DeError::invalid_type(value_to_unexpected(&value), &"dict: file info or directory")?
            }
        } else {
            None
        }
    }
}
*/
/*
impl<'de> Deserialize<'de> for FileTree {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if let Value::Dict(paths) = Value::deserialize(deserializer)? {
            paths.drain().map(|(name, value)| {});
        }
        unimplemented!()
    }
}*/

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use serde_bencode::Deserializer;

    use super::FileTree;

    #[test]
    fn filetree_from_bencode() {
        // From BEP-0052
        let bencode = "d4:infod9:file treed4:dir1d4:dir2d9:fileA.txtd0:d5:lengthi1024e11:pieces root32:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaeeeeeee";

        let mut deserializer = Deserializer::new(bencode.as_bytes());
        FileTree::deserialize(&mut deserializer).unwrap();
    }
}
