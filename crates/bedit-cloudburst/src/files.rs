//! Types representing files shared by a torrent.
//!
//! [SharedFiles] is for version 1 or hybrid torrents.
//! [FileTree] is for version 2 or hybrid torrents.
//!
//! Compared to version 1 torrents, version 2 torrents may be smaller in size due to [FileTree]s deduplicating paths.

use super::{
    crypto::{
        md5::Md5,
        sha::{Sha1, Sha256},
    },
    fileattributes::TorrentFileAttributes,
};
use either::Either;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::{
    collections::{btree_map, BTreeMap, VecDeque},
    fs::File,
    iter::FusedIterator,
    marker::PhantomData,
    num::NonZeroU64,
    path::{Path, PathBuf},
    rc::Rc,
};

#[cfg(debug_assertions)]
const FILETREE_DE_TARGET: &str = "bedit_cloudburst::FileTree::deserialize";
#[cfg(debug_assertions)]
use log::{debug, error};

/// Files shared by the torrent if multiple as per meta version 1.
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct SharedFiles {
    /// File attribute such as whether the file is executable or hidden.
    #[serde(default)]
    pub attr: Option<TorrentFileAttributes>,
    /// Length of the file in bytes.
    pub length: NonZeroU64,
    /// List of UTF-8 strings consisting of subdirectory names where the last string is the file name.
    pub path: Vec<PathBuf>,
    /// Checksum for the shared file.
    #[serde(default)]
    pub md5sum: Option<Md5>,
    /// SHA1 of file to aid file deduplication.
    #[serde(default)]
    pub sha1: Option<Sha1>,
    /// Paths for symbolic links.
    #[serde(default, rename = "symlink path")]
    pub symlink_path: Option<Vec<String>>,
}

/// File info for version 2.0 torrents.
///
/// V2 torrents use a different encoding scheme for files. Files and directories are stored as a tree where
/// the leaf nodes describe files.
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct FileTreeInfo {
    /// File attribute such as whether a file is executable or hidden.
    #[serde(default)]
    pub attr: Option<TorrentFileAttributes>,
    /// Length of the file in bytes.
    pub length: NonZeroU64,
    /// Merkel tree root as a SHA256 hash.
    #[serde(default, rename = "pieces root")]
    pub pieces_root: Option<Sha256>,
}

/// A file or a directory in version 2 [FileTree]s.
///
/// # Examples
/// [FileTreeEntry] should be deserialized as part of the overall torrent parsing process.
///
/// ```
/// use bedit_cloudburst::FileTreeEntry;
/// use serde_bencode::Error;
///
/// let file_de = "d9:cat_videod0:d6:lengthi1024000000e11:pieces root32:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaeee";
/// let file_entry: FileTreeEntry = serde_bencode::from_str(file_de)?;
/// let file_se = serde_bencode::to_string(&file_entry)?;
/// assert_eq!(file_de, file_se);
///
/// # Ok::<(), Error>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(transparent)]
pub struct FileTreeEntry(
    #[serde(with = "either::serde_untagged")] pub Either<FileTreeInfo, FileTree>,
);

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(not(debug_assertions), derive(Deserialize))]
#[serde(transparent)]
pub struct FileTree {
    pub node: BTreeMap<String, FileTreeEntry>,
}

impl<'iter> FileTree {
    pub fn iter_dfs(&'iter self) -> FileTreeDepthFirstIter<'iter> {
        //let mut iters = VecDeque::new();
        //iters.push_front(self.node.iter());
        let iters: VecDeque<_> = [(vec!["./"], self.node.iter())].into();

        FileTreeDepthFirstIter {
            tree: PhantomData,
            iters,
        }
    }
}

#[cfg(debug_assertions)]
impl<'de> Deserialize<'de> for FileTree {
    // This impl is primarily for better error logs during deserialization.
    // [bedit_cloudburst::Info] is deserialized by matching till a valid variant is found.
    // However, the error from the deserialized types is consumed leading to an entirely non-descriptive message: "data did not match any variant of untagged enum Info"
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        debug!(target: FILETREE_DE_TARGET, "Deserializing `FileTree`.");
        let node = match BTreeMap::<String, FileTreeEntry>::deserialize(deserializer) {
            Ok(node) => node,
            Err(e) => {
                error!(
                    target: FILETREE_DE_TARGET,
                    "Failed deserializing `FileTree`\nError:{e}"
                );

                return Err(e);
            }
        };

        debug!(
            target: FILETREE_DE_TARGET,
            "`FileTree` root length: {}",
            node.len()
        );
        Ok(FileTree { node })
    }
}

/// A view of a file yielded by a tree iterator.
///
/// Paths are represented as invididual components stored in a vector.
/// The path: ./alienwarpowers/models/dumbbert.mdl
/// Would be represented as:
///
/// ```rust
/// use crate::{FileTreePathView, FileTreeInfo};
///
/// let dumbbert = FileTreePathView {
///        directory: vec!["./", "alienwarpowers", "models"],
///        name: "dumbbert.mdl",
///        file_info: &FileTreeInfo {
///            attr: None,
///            length: 1.try_into().unwrap(),
///            pieces_root: None,
///        },
///    };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileTreePathView<'iter> {
    /// Directory path components.
    pub directory: Vec<&'iter str>,
    /// File name.
    pub name: &'iter str,
    /// Length and hashes for the file.
    pub file_info: &'iter FileTreeInfo,
}

/// Depth first iterator for [FileTree].
pub struct FileTreeDepthFirstIter<'iter> {
    // The iterator returns references to strings held by an instance of FileTree, but it doesn't need to own it.
    tree: PhantomData<&'iter FileTree>,
    // Holds iterators produced by traversing the FileTree as well as keeps directory state (see implementation).
    iters: VecDeque<(
        Vec<&'iter str>,
        btree_map::Iter<'iter, String, FileTreeEntry>,
    )>,
}

impl<'iter> Iterator for FileTreeDepthFirstIter<'iter> {
    type Item = FileTreePathView<'iter>;

    fn next(&mut self) -> Option<Self::Item> {
        let (directory, mut cur_iter) = self.iters.pop_front()?;

        match cur_iter.next() {
            Some((name, entry)) => match &entry.0 {
                Either::Left(file_info) => {
                    // I can't return a slice because it's owned by the iterator.
                    let directory_view = directory.clone();
                    // The iterator yielded a file therefore it needs to be checked again on the next call to...next().
                    self.iters.push_front((directory, cur_iter));

                    Some(FileTreePathView {
                        directory: directory_view,
                        name: name.as_str(),
                        file_info,
                    })
                }
                Either::Right(dir) => {
                    // The iterator yielded a directory so the NEXT directory is the old directory with the next path name appended.
                    let mut directory = directory.clone();
                    directory.push(name.as_str());

                    // As this is depth first, the next iterator is the next directory rather than exhausting the current iterator.
                    self.iters.push_front((directory, dir.node.iter()));
                    // Call next() to yield the next file. This is recursive and can cause a Stack Overflow with a malicious torrent.
                    // So uh, fix it later.
                    self.next()
                }
            },
            // Current iterator has been expended; now traverse backward down the tree.
            None => self.next(),
        }
    }
}

impl FusedIterator for FileTreeDepthFirstIter<'_> {}

#[cfg(test)]
mod tests {
    use super::{FileTree, FileTreeEntry, FileTreeInfo, FileTreePathView};
    use either::Either;
    use serde::{Deserialize, Serialize};
    use serde_bencode::Deserializer;

    // Convenience function to return a new FileTreeEntry that's a file.
    fn new_file<S>(name: S) -> (String, FileTreeEntry)
    where
        S: Into<String>,
    {
        (
            name.into(),
            FileTreeEntry(Either::Left(FileTreeInfo {
                attr: None,
                length: 1.try_into().unwrap(),
                pieces_root: None,
            })),
        )
    }

    // Convenience function to return a new FileTreeEntry that's a directory.
    fn new_dir(name: &str, entries: Vec<(String, FileTreeEntry)>) -> (String, FileTreeEntry) {
        (
            name.to_owned(),
            FileTreeEntry(Either::Right(FileTree {
                node: entries.into_iter().collect(),
            })),
        )
    }

    // A FileTree consisting of multiple files and a few nested directories.
    fn multiple_files_tree() -> FileTree {
        FileTree {
            node: [
                new_file("alienwarpowers"),
                new_file("alienwarpowers.exe"),
                new_dir(
                    "assets",
                    vec![new_dir(
                        "audio",
                        vec![
                            new_dir(
                                "music",
                                (0..3)
                                    .map(|n| new_file(format!("jon_music{n}.mp3")))
                                    .collect(),
                            ),
                            new_dir(
                                "sound",
                                (0..3)
                                    .map(|n| new_file(format!("soundlol{n}.wav")))
                                    .collect(),
                            ),
                        ],
                    )],
                ),
                new_dir(
                    "media",
                    vec![new_file("manual.pdf"), new_file("aliens.pdf")],
                ),
            ]
            .into_iter()
            .collect(),
        }
    }

    fn pls_equal(view_one: FileTreePathView<'_>, name: &str, directory: Vec<&str>) {
        let (file_name, file_info) = new_file(name);
        let file_info = file_info.0.unwrap_left();

        let view_two = FileTreePathView {
            directory,
            name: &file_name,
            file_info: &file_info,
        };

        assert_eq!(view_one, view_two)
    }

    #[derive(Deserialize, Serialize)]
    struct OuterTest {
        #[allow(unused)]
        info: TestInfo,
    }

    #[derive(Deserialize, Serialize)]
    struct TestInfo {
        #[allow(unused)]
        #[serde(rename = "file tree")]
        file_tree: FileTree,
    }

    // Copied directly from BEP-0052 with a typo fixed.
    // The original info dict has d5:length but it should be d6:length
    const BENCODE: &str = "d4:infod9:file treed4:dir1d4:dir2d9:fileA.txtd0:d6:lengthi1024e11:pieces root32:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaeeeeeee";

    #[test]
    fn filetree_bencode_roundtrip() -> Result<(), serde_bencode::Error> {
        // Deserialize
        let mut deserializer = Deserializer::new(BENCODE.as_bytes());
        let info = OuterTest::deserialize(&mut deserializer)?;

        // Serialize
        let info_se = serde_bencode::to_string(&info)?;
        assert_eq!(BENCODE, info_se);

        Ok(())
    }

    #[test]
    fn filetree_single_iter_depth() {
        let tree = FileTree {
            node: [new_file("raees_buggy_code.js")].into_iter().collect(),
        };

        let tree_view: Vec<_> = tree.iter_dfs().collect();
        assert_eq!(tree_view.len(), 1);

        let (file_name, file_info) = new_file("raees_buggy_code.js");
        let file_info = file_info.0.unwrap_left();

        let raees_view = FileTreePathView {
            directory: vec!["./"],
            name: &file_name,
            file_info: &file_info,
        };
        assert_eq!(
            tree_view
                .into_iter()
                .next()
                .expect("Expected Raees' buggy code."),
            raees_view
        );
    }

    #[test]
    fn filetree_multiple_iter_depth() {
        let tree = multiple_files_tree();
        let mut tree_view = tree.iter_dfs();

        pls_equal(tree_view.next().unwrap(), "alienwarpowers", vec!["./"]);
    }

    #[test]
    fn filetree_dirs_o_fun() {
        //(0..100).fold()
    }
}
