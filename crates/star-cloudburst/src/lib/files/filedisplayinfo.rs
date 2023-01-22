use super::{FileTree, FileTreeDepthFirstIter, FlatFile, MetaV1FileRepr};
use crate::metainfo::MetaV1;
use std::{
    iter::{self, FusedIterator, Map, Once},
    marker::PhantomData,
    num::NonZeroU64,
    slice::Iter,
};

/// Path, name, and length of a file shared by a torrent (meta info agnostic).
#[derive(Debug, Clone)]
pub struct FileDisplayInfo<'file> {
    pub file_path: Vec<&'file str>,
    pub name: &'file str,
    pub length: NonZeroU64,
}

/// Iterators that yield [FileDisplayInfo] based on the meta info dictionary version.
pub(crate) enum FileDisplayInfoBranches<'iter> {
    /// Meta info version 1: single file
    MetaV1Once(Once<FileDisplayInfo<'iter>>),
    /// Meta info version 1: multiple files
    #[allow(clippy::complexity)]
    MetaV1Multi(Map<Iter<'iter, FlatFile>, &'iter dyn Fn(&FlatFile) -> FileDisplayInfo>),
    /// Meta info version 2: single or multiple files
    MetaV2(PathViewIntoDisplayInfoIter<'iter>),
}

/// Transform a collection of [FlatFile] or a [FileTree] into iterators of [FileDisplayInfo] without cloning owned values.
pub(crate) trait AsFileDisplayInfo {
    fn as_file_display(&self) -> FileDisplayInfoBranches<'_>;
}

impl AsFileDisplayInfo for MetaV1 {
    fn as_file_display(&self) -> FileDisplayInfoBranches<'_> {
        match &self.files {
            &MetaV1FileRepr::Single(length) => {
                FileDisplayInfoBranches::MetaV1Once(iter::once(FileDisplayInfo {
                    file_path: vec![],
                    name: self.name.as_str(),
                    length,
                }))
            }
            MetaV1FileRepr::Multiple(files) => {
                FileDisplayInfoBranches::MetaV1Multi(files.iter().map(&|flat_file| {
                    let mut file_path: Vec<_> = flat_file.path.iter().map(String::as_str).collect();
                    // The last string is the name of the file.
                    let name = file_path.remove(file_path.len() - 1);

                    FileDisplayInfo {
                        file_path,
                        name,
                        length: flat_file.length,
                    }
                }))
            }
        }
    }
}

/// Iterator to map [FileTreePathView] => [FileDisplayInfo].
///
/// The iterator's lifetime is a subset of [FileTree]'s lifetime.
/// [FileTreePathView]'s (which is yielded by the depth first iter) lifetime is a subset of
/// the tree from which it borrows the path and file names. Therefore, [FileDisplayInfo]'s lifetime
/// is a subset of that lifetime too since I'm just transferring ownership of the borrows.
pub(crate) struct PathViewIntoDisplayInfoIter<'iter> {
    phantom: PhantomData<FileTree>,
    iter: FileTreeDepthFirstIter<'iter>,
}

impl<'iter> Iterator for PathViewIntoDisplayInfoIter<'iter> {
    type Item = FileDisplayInfo<'iter>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let view = self.iter.next()?;
        Some(FileDisplayInfo {
            file_path: view.directory.clone(),
            name: view.name,
            length: view.file_info.length,
        })
    }
}

impl FusedIterator for PathViewIntoDisplayInfoIter<'_> {}

impl AsFileDisplayInfo for FileTree {
    #[inline]
    fn as_file_display(&self) -> FileDisplayInfoBranches<'_> {
        FileDisplayInfoBranches::MetaV2(PathViewIntoDisplayInfoIter {
            phantom: PhantomData,
            iter: self.iter_dfs(),
        })
    }
}

pub struct FileDisplayInfoIter<'iter> {
    pub(crate) branches: FileDisplayInfoBranches<'iter>,
}

impl<'iter> Iterator for FileDisplayInfoIter<'iter> {
    type Item = FileDisplayInfo<'iter>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.branches {
            FileDisplayInfoBranches::MetaV1Once(iter) => iter.next(),
            FileDisplayInfoBranches::MetaV1Multi(iter) => iter.next(),
            FileDisplayInfoBranches::MetaV2(iter) => iter.next(),
        }
    }
}

impl FusedIterator for FileDisplayInfoIter<'_> {}
