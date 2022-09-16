//! Types representing files shared by a torrent.
//!
//! [SharedFiles] is for version 1 or hybrid torrents.
//! [FileTree] is for version 2 or hybrid torrents.
//!
//! Compared to version 1 torrents, version 2 torrents may be smaller in size due to [FileTree]s deduplicating paths.

use super::fileattributes::TorrentFileAttributes;
use either::Either;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use serde_with::skip_serializing_none;
use std::{collections::BTreeMap, num::NonZeroU64};

/// Files shared by the torrent if multiple as per meta version 1.
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

/// A file or a directory in version 2 [FileTree]s.
///
/// # Examples
/// [FileTreeEntry] should be deserialized as part of the overall torrent parsing process.
///
/// ```
/// use bedit_torrent::FileTreeEntry;
/// use serde_bencode::Error;
///
/// let file_de = "d9:cat_videod0:d6:lengthi1024000000e11:pieces root32:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaeee";
/// let file_entry: FileTreeEntry = serde_bencode::from_str(file_de)?;
/// let file_se = serde_bencode::to_string(&file_entry)?;
/// assert_eq!(file_de, file_se);
///
/// # Ok::<(), Error>(())
/// ```
#[derive(Debug, Deserialize, Serialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct FileTreeEntry(
    #[serde(with = "either::serde_untagged")] pub Either<FileTreeInfo, FileTree>,
);

/*
struct FileTreeEntryTemp(Either<FileTreeInfo, HashMap<String, Value>>);

impl TryFrom<Value> for FileTreeEntryTemp {
    type Error = DeError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Dict(map) = value {
            map.try_into()
        } else {
            Err(DeError::invalid_type(
                value_to_unexpected(&value),
                &"dictionary of files and directories",
            ))
        }
    }
}

impl TryFrom<HashMap<String, Value>> for FileTreeEntryTemp {
    type Error = DeError;

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
                    ))
                }
            }
            _ => Ok(FileTreeEntryTemp(Either::Right(value))),
        }
    }
}

*/

#[derive(Debug, Deserialize, Serialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct FileTree {
    pub node: BTreeMap<String, FileTreeEntry>,
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
#[cfg(test)]
mod tests {
    use super::FileTree;
    use serde::Deserialize;
    use serde_bencode::Deserializer;

    #[derive(Deserialize)]
    struct OuterTest {
        #[allow(unused)]
        info: TestInfo,
    }

    #[derive(Deserialize)]
    struct TestInfo {
        #[allow(unused)]
        #[serde(rename = "file tree")]
        file_tree: FileTree,
    }

    #[test]
    fn filetree_from_bencode() {
        // Copied directly from BEP-0052 with a typo fixed.
        // The original info dict has d5:length but it should be d6:length
        let bencode = "d4:infod9:file treed4:dir1d4:dir2d9:fileA.txtd0:d6:lengthi1024e11:pieces root32:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaeeeeeee";

        let mut deserializer = Deserializer::new(bencode.as_bytes());
        OuterTest::deserialize(&mut deserializer).unwrap();
    }
}
