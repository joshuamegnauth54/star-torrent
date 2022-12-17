use super::{FileTree, FileTreeDepthFirstIter, FileTreePathView, FlatFile, MetaV1FileRepr};
use crate::metainfo::MetaV1;
use std::{
    iter::{self, Map, Once},
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

pub enum FileDisplayInfoBranches<'iter> {
    /// Meta info version 1: single file
    MetaV1Once(Once<FileDisplayInfo<'iter>>),
    /// Meta info version 1: multiple files
    MetaV1Multi(Map<Iter<'iter, FlatFile>, &'iter dyn Fn(&FlatFile) -> FileDisplayInfo>),
    /// Meta info version 2: single or multiple files
    MetaV2(
        Map<
            FileTreeDepthFirstIter<'iter>,
            &'iter dyn Fn(FileTreePathView) -> FileDisplayInfo<'iter>,
        >,
    ),
}

fn flatfile_multi_to_filedisplay(flat_file: &FlatFile) -> FileDisplayInfo {
    let mut file_path: Vec<_> = flat_file.path.iter().map(String::as_str).collect();
    let name = file_path.remove(file_path.len() - 1);

    FileDisplayInfo {
        file_path,
        name,
        length: flat_file.length,
    }
}

/// Transform a collection of [FlatFile] or a [FileTree] into iterators of [FileDisplayInfo]
pub trait IntoFileDisplayInfo {
    fn into_file_display(&self) -> FileDisplayInfoBranches<'_>;
}

impl IntoFileDisplayInfo for MetaV1 {
    fn into_file_display(&self) -> FileDisplayInfoBranches<'_> {
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

impl IntoFileDisplayInfo for FileTree {
    // The anonymous lifetime is a subset of [FileTree]'s lifetime. [FileTreePathView]'s lifetime is a subset of
    // the tree from which it borrows the path and file names. Therefore, [FileDisplayInfo]'s lifetime
    // is a subset of that lifetime too since I'm just transferring ownership of the borrows.
    fn into_file_display(&self) -> FileDisplayInfoBranches<'_> {
        FileDisplayInfoBranches::MetaV2(self.iter_dfs().map(&|view| FileDisplayInfo {
            file_path: view.directory.clone(),
            name: view.name,
            length: view.file_info.length,
        }))
    }
}

pub struct FileDisplayInfoIter<'iter, I>
where
    I: Iterator<Item = FileDisplayInfo<'iter>>,
{
    iter: I,
}

use crate::metainfo::MetaInfo;
impl<'iter, I> FileDisplayInfoIter<'iter, I>
where
    I: Iterator<Item = FileDisplayInfo<'iter>>,
{
    fn file_display_info_iter(info: &MetaInfo) -> I {
        match info {
            MetaInfo::MetaV1(v1) => {
                unimplemented!()
            }
            MetaInfo::MetaV2(v2) => {
                unimplemented!()
            }
            MetaInfo::Hybrid(hybrid) => {
                unimplemented!()
            }
        }
    }
}

/*impl IntoFileDisplayInfo for crate::metainfo::MetaV1 {
    fn into_file_display<'file>(&'file self) -> FileDisplayInfo<'file> {
        if let Some(files) = &self.files {
            files.iter().map(|file| {
                // let dir = std::iter::once(info.name.as_str()).chain(file.path.iter().map(|p|)

                FileDisplayInfo { file_path: vec![] }
            })
        } else if let Some(file_length) = self.length {
            FileDisplayInfo {
                iter: std::iter::once(FileDisplayInfo {
                    file_path: vec![],
                    name: &info.name,
                    length: file_length.get(),
                }),
            }
        }
    }
}
*/
