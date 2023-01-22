use crate::{
    crypto::{md5::Md5, sha1::Sha1},
    files::fileattributes::TorrentFileAttributes,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::{num::NonZeroU64, path::PathBuf};

/// Files shared by the torrent if multiple as per meta version 1.
/// Meta version 1 represents files in a flattened structure where `path` represents the full
/// path of the file including the directory and the name. Files in the same directory repeat the directory
/// strings per file.
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(debug_assertions, serde(deny_unknown_fields))]
pub struct FlatFile {
    /// File attribute such as whether the file is executable or hidden.
    #[serde(default)]
    pub attr: Option<TorrentFileAttributes>,
    /// Length of the file in bytes.
    pub length: NonZeroU64,
    /// List of UTF-8 strings consisting of subdirectory names where the last string is the file name.
    pub path: Vec<String>,
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

/// Does this torrent share multiple files or a single file?
///
/// Meta version 1 represents multiple files with a list of [FlatFile].
/// Single file torrents only include a `length` field with `name` indicating the suggested name of the file.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(tag = "untagged")]
pub enum MetaV1FileRepr {
    #[serde(rename = "files")]
    Multiple(Vec<FlatFile>),
    #[serde(rename = "length")]
    Single(NonZeroU64),
}

#[cfg(test)]
mod tests {
    use super::{FlatFile, MetaV1FileRepr};
    use serde::{Deserialize, Serialize};
    use serde_test::{assert_de_tokens, assert_tokens, Token};

    #[derive(Deserialize, Serialize, Debug, PartialEq)]
    struct LameV1Files {
        files: MetaV1FileRepr,
    }

    fn fake_file() -> FlatFile {
        FlatFile {
            attr: None,
            length: 42.try_into().unwrap(),
            path: (0..10)
                .flat_map(|n| ["scripts".into(), "ai".into(), format!("raees_{n}.py")])
                .collect(),
            md5sum: None,
            sha1: None,
            symlink_path: None,
        }
    }

    #[test]
    fn yield_one_metav1_de() {
        let files = LameV1Files {
            files: MetaV1FileRepr::Single(42.try_into().unwrap()),
        };

        assert_de_tokens(
            &files,
            &[
                Token::Struct {
                    name: "LameV1Files",
                    len: 1,
                },
                Token::Str("files"),
                Token::Enum {
                    name: "MetaV1FileRepr",
                },
                Token::Str("untagged"),
                Token::Str("length"),
                Token::U64(42),
                Token::StructEnd,
            ],
        )
    }

    #[test]
    fn yield_multi_metav2_de() {
        let files = LameV1Files {
            files: MetaV1FileRepr::Multiple((0..10).map(|_| fake_file()).collect()),
        };

        assert_tokens(
            &files,
            &[
                Token::Struct {
                    name: "LameV1Files",
                    len: 1,
                },
                Token::Str("files"),
                Token::Enum {
                    name: "MetaV1FileRepr"
                },
                Token::Str("untagged"),
                Token::Str("files"),
                Token::StructEnd,
            ],
        );
    }
}
